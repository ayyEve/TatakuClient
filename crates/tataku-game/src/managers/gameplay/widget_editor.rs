use crate::prelude::*;
use crate::prelude::ui::*;
use std::sync::mpsc::TryRecvError;

/// dialog
pub struct GameplayWidgetEditor {
    // builders: Vec<GameplayWidgetBuilder>,
    sender: Sender<GameplayWidgetAction>,
    receiver: Arc<Mutex<Receiver<GameplayWidgetEvent>>>,

    widgets: Vec<WidgetState>,

    click_action: ClickAction,
    context_menu: Option<ContextMenu>,

    click_pos: Option<ClickHoldData>,

    node: Box<dyn Widget>,
}
impl GameplayWidgetEditor {
    pub fn new(
        // builders: Vec<GameplayWidgetBuilder>,
        widgets: &[GameplayWidgetContainer],
        sender: Sender<GameplayWidgetAction>,
        receiver: Receiver<GameplayWidgetEvent>,
    ) -> Self {
        let widgets = widgets
            .iter()
            .map(WidgetState::new)
            .collect();

        Self {
            // builders,
            sender,
            receiver: Arc::new(Mutex::new(receiver)),

            widgets,
            click_action: ClickAction::None,
            context_menu: None,
            click_pos: None,

            node: EmptyWidget::new_boxed(),
        }
    }

    fn close(
        &self,
        actions: &mut ActionQueue,
    ) {
        let _ = self.sender.send(
            GameplayWidgetAction {
                target: String::new(),
                action: GameplayWidgetActionType::Done,
            }
        );

        actions.push(UiAction::new(
            self.node_id(),
            DialogAction::Close,
        ));
    }

    fn send(
        &self, 
        action: GameplayWidgetAction,
        actions: &mut ActionQueue
    ) {
        if self.sender.send(action).is_err() {
            actions.push(UiAction::new(
                self.node_id(),
                DialogAction::Close,
            ));
        }
    }

    pub fn handle_event(
        &mut self, 
        shell: &mut UpdateShell,
        event: GameplayWidgetEvent,
    ) {
        match event.action {
            GameplayWidgetEventType::Update { bounds } => {
                let Some(target) = event.target else { return };
                for i in self.widgets.iter_mut() {
                    if i.name != target { continue }
                    i.bounds = bounds;
                    break;
                }
            }
            GameplayWidgetEventType::Close => {
                self.close(shell.actions);
            }
        }
    }
    fn get_selected(&mut self) -> Option<&mut WidgetState> {
        self.widgets
            .iter_mut()
            .find(|w| w.selected)
    }

    fn build_context_menu(
        &self,
        shell: &mut InputShell,
        selected: usize,
    ) -> ContextMenu {
        let selected = &self.widgets[selected];

        let mut options = Vec::new();

        options.push(ContextMenuOption::new(
            selected.name.clone(), 
            ContextMenuOptionType::TextOnly
        ));

        // align
        options.push({
            const ALL_ALIGN: &[(&str, Alignment)] = &[
                ("Top Left", Alignment::TOP_LEFT), ("Top Middle", Alignment::TOP_CENTER), ("Top Right", Alignment::TOP_RIGHT),
                ("Center Left", Alignment::CENTER_LEFT), ("Center", Alignment::CENTER), ("Center Right", Alignment::CENTER_RIGHT),
                ("Bottom Left", Alignment::BOTTOM_LEFT), ("Bottom Middle", Alignment::BOTTOM_MIDDLE), ("Bottom Right", Alignment::BOTTOM_RIGHT)
            ];

            let mut align_builder = ContextMenuBuilder::new();
            for &(label, align) in ALL_ALIGN {
                align_builder.add_option(ContextMenuOption::new(
                    if align == selected.layout.align {
                        Cow::Owned(format!("{label} ✓"))
                    } else {
                        Cow::Borrowed(label)
                    },
                    ContextMenuOptionType::Action(Message::new(
                        shell.owner, 
                        "align",
                        MessageValue::Custom(Arc::new(align))
                    ).into())
                ));
            }

            ContextMenuOption::new("Alignment", align_builder)
        });

        // visible
        options.push(ContextMenuOption::new(
            if selected.layout.visible { "Visible ✓" } else { "Visible" },
            ContextMenuOptionType::Action(Message::new(
                shell.owner,
                "visible",
                MessageValue::Click
            ).into())
        ));
        // anchor
        options.push(ContextMenuOption::new(
            "Anchor",
            ContextMenuOptionType::SubMenu(
                ContextMenuBuilder::new()
                .with_option(ContextMenuOption::new(
                    "Screen",
                    ContextMenuOptionType::Action(Message::new(
                        shell.owner,
                        "anchor",
                        MessageValue::Text("screen".to_string())
                    ).into())
                ))
                .with_option(ContextMenuOption::new(
                    "Playfield",
                    ContextMenuOptionType::Action(Message::new(
                        shell.owner,
                        "anchor",
                        MessageValue::Text("playfield".to_string())
                    ).into())
                ))
                .with_option(ContextMenuOption::new(
                    "Element",
                    ContextMenuOptionType::Action(Message::new(
                        shell.owner,
                        "anchor",
                        MessageValue::Text("element".to_string())
                    ).into())
                ))
            )
        ));

        fn make_relative_align(
            current: GameplayWidgetAlign,
            owner: MessageOwner,
        ) -> ContextMenuOptionType {
            const ALL_INNER_ALIGN: &[(&str, GameplayWidgetAlign)] = &[
                ("Inside", GameplayWidgetAlign::Inside),
                ("Above", GameplayWidgetAlign::Above),
                ("Below", GameplayWidgetAlign::Below),
                ("Left", GameplayWidgetAlign::Left),
                ("Right", GameplayWidgetAlign::Right),
            ];
            
            let mut builder = ContextMenuBuilder::new();
            for &(label, a) in ALL_INNER_ALIGN {
                builder.add_option(ContextMenuOption::new(
                    if a == current {
                        Cow::Owned(format!("{label} ✓"))
                    } else {
                        Cow::Borrowed(label)
                    }, 
                    ContextMenuOptionType::Action(
                        Message::new(
                            owner, 
                            "relative_align",
                            MessageValue::Custom(Arc::new(a))
                        ).into()
                    )
                ));
            }

            ContextMenuOptionType::SubMenu(builder)
        }

        // anchor options
        match &selected.layout.anchor {
            GameplayWidgetAnchor::Screen => {},
            GameplayWidgetAnchor::Playfield { 
                relative,
                ..
            } => {
                options.push(ContextMenuOption::new(
                    "Relative Align",
                    make_relative_align(*relative, shell.owner),
                ));
            }
            GameplayWidgetAnchor::Element { 
                relative ,
                ..
            } => {
                options.push(ContextMenuOption::new(
                    "Relative Align",
                    make_relative_align(*relative, shell.owner),
                ));
                options.push(ContextMenuOption::new(
                    "Select Parent",
                    ContextMenuOptionType::Action(Message::new(
                        shell.owner,
                        "select_parent",
                        MessageValue::Click
                    ).into()),
                ));
            }
        }


        // reset
        options.push(ContextMenuOption::new(
            "Reset",
            ContextMenuOptionType::Action(Message::new(
                shell.owner,
                "reset",
                MessageValue::Click,
            ).into())
        ));
        // reset to default
        options.push(ContextMenuOption::new(
            "Default",
            ContextMenuOptionType::Action(Message::new(
                shell.owner,
                "reset_default",
                MessageValue::Click,
            ).into())
        ));

        ContextMenu::new(options, shell.mouse_pos)
    }


    fn handle_left_click(
        &mut self, 
        shell: &mut InputShell,
        clicked: Option<usize>, 
        selected: Option<usize>,
    ) {
        match self.click_action {
            ClickAction::None => {
                if let Some(clicked) = clicked {
                    self.widgets[clicked].selected = true;
                    shell.event_consumed = true;

                    self.click_pos = Some(ClickHoldData { 
                        pos: shell.mouse_pos, 
                        triggered: false, 
                        selected: clicked, 
                    });
                } else {
                    self.context_menu = None;
                }
                if let Some(selected) = selected {
                    self.widgets[selected].selected = false;
                }
            }
            ClickAction::SetAnchorElement => {
                if let Some((selected, clicked)) = selected.zip(clicked) {
                    shell.event_consumed = true;
                    if selected == clicked { return }

                    let mid = (clicked + selected) / 2;
                    let (
                        left, 
                        right
                    ) = self.widgets.split_at_mut(mid);

                    let (clicked, selected) = {
                        if selected > clicked {(
                            &mut left[clicked],
                            &mut right[selected - mid],
                        )} else {(
                            &mut right[clicked - mid],
                            &mut left[selected],
                        )}
                    };

                    match &mut selected.layout.anchor {
                        GameplayWidgetAnchor::Element { 
                            element, 
                            .. 
                        } => {
                            *element = clicked.name.clone().into();
                        },

                        _ => {
                            selected.layout.anchor = GameplayWidgetAnchor::Element { 
                                element: clicked.name.clone().into(), 
                                relative: GameplayWidgetAlign::Inside,
                            };
                        }
                    }
                }
            }
        }
    }

    fn handle_right_click(
        &mut self,
        shell: &mut InputShell,
        selected: Option<usize>,
    ) {
        let Some(selected) = selected else { return };
        self.context_menu = Some(self.build_context_menu(shell, selected));
    }
}
impl Widget for GameplayWidgetEditor {
    fn name(&self) -> CowStr { "widget_editor".into() }
    fn node_id(&self) -> NodeId { self.node.node_id() }
    
    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        self.node = Container::new(
            self.widgets
                .iter()
                .map(|w| {
                    Button::new(TextWidget::new(w.name.clone()).boxed())
                    .on_press(Message::new(
                        shell.owner, 
                        w.name.clone(), 
                        MessageValue::Click
                    ))
                    .boxed()
                })
                .collect()
        )
        .flex_direction(FlexDirection::Column)
        .scrollable(true)
        .width(FILL).height(FILL)
        .boxed();

        self.node.layout(shell)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell,
    ) {
        if let Some(menu) = self.context_menu.as_mut() {
            menu.input(event, shell);
        }

        if shell.event_consumed { return }
        match event.event {
            InputType::MouseMove(pos) => {
                if let Some(data) = &mut self.click_pos {
                    if !data.triggered 
                        && shell.mouse_pos.distance(data.pos) > 10.0
                    {
                        data.triggered = true;
                    }

                    shell.event_consumed = true;

                    if data.triggered {
                        let selected = &self.widgets[data.selected];
                        let mut layout = selected.layout.clone();
                        layout.offset += pos - data.pos;
                        self.send(
                            GameplayWidgetAction { 
                                target: selected.name.clone(), 
                                action: GameplayWidgetActionType::Move(layout),
                            }, 
                            shell.actions,
                        );
                    }

                    return
                }

                let mut hover_found = false;
                for i in self.widgets.iter_mut() {
                    if hover_found {
                        i.hover = false;
                    } else if i.bounds.contains(pos) {
                        shell.event_consumed = true;
                        hover_found = true;
                        i.hover = true;
                    } else {
                        i.hover = false;
                    }
                }
            }

            InputType::MousePress(MouseButton::Left) => {
                let mut clicked = None;
                let mut selected = None;
                for (n, i) in self
                    .widgets
                    .iter_mut()
                    .enumerate()
                {
                    if i.selected { selected = Some(n); }
                    if i.hover { clicked = Some(n); }
                
                    if selected.is_some() && clicked.is_some() { break }
                }

                self.handle_left_click(shell, clicked, selected);
            }
            InputType::MouseRelease(MouseButton::Left) => {
                self.click_pos = None;
            }

            InputType::MousePress(MouseButton::Right) => {
                let mut clicked = None;
                let mut selected = None;
                for (n, i) in self
                    .widgets
                    .iter_mut()
                    .enumerate()
                {
                    if i.selected { selected = Some(n); }
                    if i.hover { clicked = Some(n); }
                    if selected.is_some() && clicked.is_some() { break }
                }

                if let Some(selected) = selected {
                    self.widgets[selected].selected = false;
                }
                if let Some(clicked) = clicked {
                    self.widgets[clicked].selected = true;
                }

                self.handle_right_click(shell, clicked);
            }
            
            _ => {}
        }
    }

    fn draw(&self, shell: &mut DrawShell) {
        for i in self.widgets.iter() {
            i.draw(shell);
        }

        self.node.draw(shell);
    }

    fn draw_overlay(&self, shell: &mut DrawShell) {
        let Some(menu) = self.context_menu.as_ref() else { return };
        menu.draw(shell);
        menu.draw_overlay(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        let receiver = self.receiver.clone();
        let receiver = receiver.lock();

        loop {
            match receiver.try_recv() {
                Ok(event) => self.handle_event(shell, event),
                Err(TryRecvError::Disconnected) => {
                    shell.actions.push(UiAction::new(
                        self.node_id(),
                        DialogAction::Close,
                    ));
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }

        if let Some(menu) = self.context_menu.as_mut() {
            menu.update(shell);
            if menu.should_close {
                self.context_menu = None;
            }
        }
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        if let Some(tag) = message.tag.as_string() {
            shell.handled = true;

            match &**tag {
                "align" => {
                    let value = *message.value.downcast::<Alignment>();
                    for i in self.widgets.iter_mut() {
                        if !i.selected { continue }
                        i.layout.align = value;

                        let action = GameplayWidgetAction { 
                            target: i.name.clone(), 
                            action: GameplayWidgetActionType::Move(i.layout.clone()),
                        };

                        self.send(action, shell.actions);
                        break;
                    }
                }
                
                "anchor" => {
                    let Some(value) = message.value.as_text_ref() 
                    else { return };
                    
                    use GameplayWidgetAnchor as Anchor;
                    for i in self.widgets.iter_mut() {
                        if !i.selected { continue }
                        let mut send_update = false;
                        
                        match (&**value, &mut i.layout.anchor) {
                            // dont change anything if the incoming type is already correct
                            ("element", Anchor::Element {..}) => break,
                            ("playfield", Anchor::Playfield {..}) => break,

                            ("screen", a) => {
                                *a = Anchor::Screen;
                                send_update = true;
                            },

                            ("element", a) => { 
                                *a = Anchor::Element { 
                                    element: Cow::Borrowed(""),
                                    relative: GameplayWidgetAlign::Inside,
                                };
                            },

                            ("playfield", a) => {
                                *a = Anchor::Playfield { 
                                    saved_size: None,
                                    relative: GameplayWidgetAlign::Inside,
                                };
                                send_update = true;
                            }

                            _ => {}
                        };

                        if send_update {
                            let action = GameplayWidgetAction { 
                                target: i.name.clone(), 
                                action: GameplayWidgetActionType::Move(i.layout.clone()),
                            };
                            self.send(action, shell.actions);
                        }

                        break;
                    }
                }

                "relative_align" => {
                    let value = *message
                        .value.downcast::<GameplayWidgetAlign>();

                    for i in self.widgets.iter_mut() {
                        if !i.selected { continue }
                        match &mut i.layout.anchor {
                            GameplayWidgetAnchor::Screen => {},
                            GameplayWidgetAnchor::Element { 
                                relative,
                                ..
                            } | GameplayWidgetAnchor::Playfield { 
                                relative,
                                ..
                            } => {
                                *relative = value;
                            }
                        }
                        let action = GameplayWidgetAction { 
                            target: i.name.clone(), 
                            action: GameplayWidgetActionType::Move(i.layout.clone()),
                        };

                        self.send(action, shell.actions);

                        break;
                    }
                }

                "select_parent" => {
                    self.click_action = ClickAction::SetAnchorElement;
                }

                "reset" => {
                    if let Some(selected) = self.get_selected() {
                        selected.layout = selected.original_layout.clone();
                        let action = GameplayWidgetAction { 
                            target: selected.name.clone(), 
                            action: GameplayWidgetActionType::Move(
                                selected.original_layout.clone()
                            ),
                        };

                        self.send(
                            action,
                            shell.actions
                        );
                    }
                }
                
                "reset_default" => {
                    if let Some(selected) = self.get_selected() {
                        selected.layout = selected.default_layout.clone();
                        let action = GameplayWidgetAction { 
                            target: selected.name.clone(), 
                            action: GameplayWidgetActionType::Move(
                                selected.default_layout.clone()
                            ),
                        };

                        self.send(
                            action,
                            shell.actions
                        );
                    }
                }

                _ => {
                    shell.handled = false;
                }
            }
        }

        if let Some(menu) = self.context_menu.as_mut() {
            menu.handle_message(message, shell);
        }
    }
}

enum ClickAction {
    None,
    SetAnchorElement,
}

struct WidgetState {
    name: String,
    layout: GameplayWidgetLayout,
    original_layout: GameplayWidgetLayout,
    default_layout: GameplayWidgetLayout,

    hover: bool,
    selected: bool,
    bounds: Bounds,
}
impl WidgetState {
    fn new(widget: &GameplayWidgetContainer) -> Self {
        Self {
            name: widget.element_name.clone(),
            hover: false,
            selected: false,
            layout: widget.layout.clone(),
            original_layout: widget.layout.clone(),
            default_layout: widget.default_layout.clone(),

            bounds: widget.get_bounds(),
        }
    }

    fn draw(&self, shell: &mut DrawShell) {
        if self.hover {
            shell.list.push(Rectangle::new_bounds(
                self.bounds,
                Color::TRANSPARENT,
            ).border(Border::new(
                shell.general_theme.hover_color, 
                2.0
            )));
        } else if self.selected {
            shell.list.push(Rectangle::new_bounds(
                self.bounds,
                Color::TRANSPARENT,
            ).border(Border::new(
                shell.general_theme.active_color, 
                2.0
            )));
        }
    }
}

#[derive(Default)]
struct ClickHoldData {
    pos: Vector2,
    triggered: bool,
    selected: usize,
}

// outgoing to gameplay manager
pub struct GameplayWidgetAction {
    pub target: String,
    pub action: GameplayWidgetActionType,
}
pub enum GameplayWidgetActionType {
    Add(GameplayWidgetLayout),
    Move(GameplayWidgetLayout),
    Remove,
    Done
}

// incoming from gameplay manager
pub struct GameplayWidgetEvent {
    pub target: Option<String>,
    pub action: GameplayWidgetEventType,
}

pub enum GameplayWidgetEventType {
    Close,
    Update {
        bounds: Bounds,
    }
}
