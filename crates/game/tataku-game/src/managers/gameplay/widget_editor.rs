use crate::prelude::*;
use tataku_interface::menu_widgets::context_menus::*;
use std::sync::mpsc::{ Sender, Receiver, TryRecvError };
use tataku::{
    Color,
    Bounds,
    Border,
    Vector2,
    Alignment,
};
use engine::{
    actions,
    gameplay::widgets::*
};
use ui::{
    tree::*,
    widget::*,
    message::*,
};

/// dialog
pub struct GameplayWidgetEditor {
    // builders: Vec<GameplayWidgetBuilder>,
    sender: Sender<GameplayWidgetAction>,
    receiver: Arc<Mutex<Receiver<GameplayWidgetEvent>>>,

    widgets: Vec<WidgetState>,

    click_action: ClickAction,
    context_menu: Option<ContextMenu>,

    click_pos: Option<ClickHoldData>,

    node: Box<dyn Widget<actions::Action>>,
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
        source: MessageSource,
        actions: &mut actions::ActionQueue,
    ) {
        let _ = self.sender.send(
            GameplayWidgetAction {
                target: String::new(),
                action: GameplayWidgetActionType::Done,
            }
        );

        actions.push(actions::ui::UiAction::new(
            self.node_id(),
            source,
            actions::dialog::DialogAction::Close,
        ).into());
    }

    fn send(
        &self,
        source: MessageSource,
        action: GameplayWidgetAction,
        actions: &mut actions::ActionQueue
    ) {
        if self.sender.send(action).is_err() {
            actions.push(actions::ui::UiAction::new(
                self.node_id(),
                source,
                actions::dialog::DialogAction::Close,
            ).into());
        }
    }

    pub fn handle_event(
        &mut self,
        shell: &mut UpdateShell<actions::Action>,
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
                self.close(shell.source, shell.actions);
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
        shell: &mut InputShell<actions::Action>,
        selected: usize,
    ) -> ContextMenu {
        let selected = &self.widgets[selected];

        let source = shell.source;

        let mut options = Vec::new();

        options.push(ContextMenuOption::new(
            selected.name.clone(),
            ContextMenuOptionType::TextOnly
        ));

        // align
        let selected_align = selected.layout.align;
        options.push(ContextMenuOption::new(
            "Alignment",
            ContextMenuOptionType::SubMenu(Box::new(move || {
                const ALL_ALIGN: &[(&str, Alignment)] = &[
                    ("Top Left", Alignment::TOP_LEFT), ("Top Middle", Alignment::TOP_CENTER), ("Top Right", Alignment::TOP_RIGHT),
                    ("Center Left", Alignment::CENTER_LEFT), ("Center", Alignment::CENTER), ("Center Right", Alignment::CENTER_RIGHT),
                    ("Bottom Left", Alignment::BOTTOM_LEFT), ("Bottom Middle", Alignment::BOTTOM_MIDDLE), ("Bottom Right", Alignment::BOTTOM_RIGHT)
                ];

                let mut options = Vec::new();

                for &(label, align) in ALL_ALIGN {
                    let message = Box::new(move || Some(Message::new(
                        source,
                        "align",
                        None,
                        Box::new(align),
                    )));

                    options.push(ContextMenuOption::new(
                        if align == selected_align {
                            Cow::Owned(format!("{label} ✓"))
                        } else {
                            Cow::Borrowed(label)
                        },
                        ContextMenuAction::Callback(message)
                    ));
                }

                ContextMenu::new(options, Vector2::ZERO)
            }))
        ));

        // visible
        options.push(ContextMenuOption::new(
            if selected.visible { "Visible ✓" } else { "Visible" },
            ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                source,
                "visible",
                None,
                Box::new(()),
            ))))
        ));
        // anchor
        options.push(ContextMenuOption::new(
            "Anchor",
            ContextMenuOptionType::SubMenu(Box::new(move || ContextMenu::new(
                vec![
                    ContextMenuOption::new(
                        "Screen",
                        ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                            source,
                            "anchor",
                            None,
                            Box::new("screen".to_string()),
                        ))))
                    ),
                    ContextMenuOption::new(
                        "Playfield",
                        ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                            source,
                            "anchor",
                            None,
                            Box::new("playfield".to_string()),
                        ))))
                    ),
                    ContextMenuOption::new(
                        "Element",
                        ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                            source,
                            "anchor",
                            None,
                            Box::new("element".to_string())
                        ))))
                    ),
                ],
                Vector2::ZERO
            )))
        ));

        fn make_side(
            current: Side,
            tag: &'static str,
            source: MessageSource,
        ) -> ContextMenuOptionType {
            let menu = Box::new(move || {
                const ALL_INNER_ALIGN: &[(&str, Side)] = &[
                    ("Inside", Side::Inside),
                    ("Outside", Side::Outside),
                ];

                let mut options = Vec::default();
                for &(label, a) in ALL_INNER_ALIGN {
                    options.push(ContextMenuOption::new(
                        if a == current {
                            Cow::Owned(format!("{label} ✓"))
                        } else {
                            Cow::Borrowed(label)
                        },
                        ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                            source,
                            tag,
                            None,
                            Box::new(a)
                        ))))
                    ));
                }

                ContextMenu::new(options, Vector2::ZERO)
            });

            ContextMenuOptionType::SubMenu(menu)
        }

        // anchor options
        match &selected.layout.anchor {
            GameplayWidgetAnchor::Screen => {},
            GameplayWidgetAnchor::Playfield {
                horizontal_side,
                vertical_side,
            } => {
                options.push(ContextMenuOption::new(
                    "Horizontal Side",
                    make_side(*horizontal_side, "horizontal_side", shell.source),
                ));
                options.push(ContextMenuOption::new(
                    "Vertical Side",
                    make_side(*vertical_side, "vertical_side", shell.source),
                ));
            }
            GameplayWidgetAnchor::Element {
                horizontal_side,
                vertical_side,
                ..
            } => {
                options.push(ContextMenuOption::new(
                    "Horizontal Side",
                    make_side(*horizontal_side, "horizontal_side", shell.source),
                ));
                options.push(ContextMenuOption::new(
                    "Vertical Side",
                    make_side(*vertical_side, "vertical_side", shell.source),
                ));
                options.push(ContextMenuOption::new(
                    "Select Parent",
                    ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                        source,
                        "select_parent",
                        None,
                        Box::new(())
                    ))))
                ));
            }
        }


        // reset
        options.push(ContextMenuOption::new(
            "Reset",
            ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                source,
                "reset",
                None,
                Box::new(()),
            ))))
        ));
        // reset to default
        options.push(ContextMenuOption::new(
            "Default",
            ContextMenuAction::Callback(Box::new(move || Some(Message::new(
                source,
                "reset_default",
                None,
                Box::new(()),
            ))))
        ));

        ContextMenu::new(options, shell.mouse_pos)
    }


    fn handle_left_click(
        &mut self,
        shell: &mut InputShell<actions::Action>,
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
                                horizontal_side: Side::Inside,
                                vertical_side: Side::Inside,
                            };
                        }
                    }
                }
            }
        }
    }

    fn handle_right_click(
        &mut self,
        shell: &mut InputShell<actions::Action>,
        selected: Option<usize>,
    ) {
        let Some(selected) = selected else { return };
        self.context_menu = Some(self.build_context_menu(shell, selected));
    }
}
impl Widget<actions::Action> for GameplayWidgetEditor {
    fn name(&self) -> CowStr { "widget_editor".into() }
    fn node_id(&self) -> NodeId { self.node.node_id() }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        let a = self.widgets
            .iter()
            .map(|w| format!(r#"
                <button>
                    <action>
                    Message::new(
                        shell.owner,
                        w.name.clone(),
                        MessageValue::Click,
                    )
                    </action>
                    <element>
                        <text>
                            <text text="{}" />
                        </text>
                    </element>
                </button>
            "#, w.name))
            .collect::<Vec<_>>()
            .join("");
        let list_str = format!(r#"
            <column
                scollable="true"
                style="width: fill; height: fill"
            >
                {a}
            </column>
        "#);

        self.node = quick_xml::de::from_str::<interface::Element>(&list_str)
            .unwrap()
            .build()
            .boxed();

        // self.node = Container::new(
        //     self.widgets
        //         .iter()
        //         .map(|w| {
        //             Button::new(TextWidget::new(w.name.clone()).boxed())
        //             .on_press(Message::new(
        //                 shell.owner,
        //                 w.name.clone(),
        //                 MessageValue::Click,
        //             ))
        //             .boxed()
        //         })
        //         .collect()
        // )
        // .flex_direction(FlexDirection::Column)
        // .scrollable(true)
        // .width(FILL)
        // .height(FILL)
        // .boxed();

        self.node.layout(shell)
    }

    fn input(
        &mut self,
        event: &input::InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        if let Some(menu) = self.context_menu.as_mut() {
            menu.input(event, shell);
        }

        if shell.event_consumed { return }
        match event.event {
            input::InputType::MouseMove(pos) => {
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
                        layout.transform.pos += pos - data.pos;
                        self.send(
                            shell.source,
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

            input::InputType::MousePress(input::MouseButton::Left) => {
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
            input::InputType::MouseRelease(input::MouseButton::Left) => {
                self.click_pos = None;
            }

            input::InputType::MousePress(input::MouseButton::Right) => {
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

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        for i in self.widgets.iter() {
            i.draw(shell);
        }

        self.node.draw(shell);
    }

    fn draw_overlay(&self, shell: &mut DrawShell<actions::Action>) {
        let Some(menu) = self.context_menu.as_ref() else { return };
        menu.draw(shell);
        menu.draw_overlay(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let receiver = self.receiver.clone();
        let receiver = receiver.lock();

        loop {
            match receiver.try_recv() {
                Ok(event) => self.handle_event(shell, event),
                Err(TryRecvError::Disconnected) => {
                    shell.actions.push(actions::ui::UiAction::new(
                        self.node_id(),
                        shell.source,
                        actions::dialog::DialogAction::Close,
                    ).into());
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
        shell: &mut MessageShell<actions::Action>,
    ) {
        shell.handled = true;

        match &*message.tag {
            "align" => {
                let Some(value) = message.value.downcast_ref::<Alignment>().copied()
                else { return };

                for i in self.widgets.iter_mut() {
                    if !i.selected { continue }
                    i.layout.align = value;

                    let action = GameplayWidgetAction {
                        target: i.name.clone(),
                        action: GameplayWidgetActionType::Move(i.layout.clone()),
                    };

                    self.send(shell.source, action, shell.actions);
                    break;
                }
            }

            "anchor" => {
                let Some(value) = message.value.downcast_ref::<String>()
                else { return };

                use engine::gameplay::widgets::GameplayWidgetAnchor as Anchor;
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
                                horizontal_side: Side::Inside,
                                vertical_side: Side::Inside,
                            };
                        },

                        ("playfield", a) => {
                            *a = Anchor::Playfield {
                                horizontal_side: Side::Inside,
                                vertical_side: Side::Inside,
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
                        self.send(shell.source, action, shell.actions);
                    }

                    break;
                }
            }

            "horizontal_side" => {
                let Some(value) = message.value
                    .downcast_ref::<Side>().copied()
                else { return};

                for i in self.widgets.iter_mut() {
                    if !i.selected { continue }
                    match &mut i.layout.anchor {
                        GameplayWidgetAnchor::Screen => {},
                        GameplayWidgetAnchor::Element {
                            horizontal_side,
                            ..
                        } | GameplayWidgetAnchor::Playfield {
                            horizontal_side,
                            ..
                        } => {
                            *horizontal_side = value;
                        }
                    }
                    let action = GameplayWidgetAction {
                        target: i.name.clone(),
                        action: GameplayWidgetActionType::Move(i.layout.clone()),
                    };

                    self.send(shell.source, action, shell.actions);

                    break;
                }
            }
            "vertical_side" => {
                let Some(value) = message.value
                    .downcast_ref::<Side>().copied()
                else { return};

                for i in self.widgets.iter_mut() {
                    if !i.selected { continue }
                    match &mut i.layout.anchor {
                        GameplayWidgetAnchor::Screen => {},
                        GameplayWidgetAnchor::Element {
                            vertical_side,
                            ..
                        } | GameplayWidgetAnchor::Playfield {
                            vertical_side,
                            ..
                        } => {
                            *vertical_side = value;
                        }
                    }
                    let action = GameplayWidgetAction {
                        target: i.name.clone(),
                        action: GameplayWidgetActionType::Move(i.layout.clone()),
                    };

                    self.send(shell.source, action, shell.actions);

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
                        shell.source,
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
                        shell.source,
                        action,
                        shell.actions
                    );
                }
            }

            _ => {
                shell.handled = false;
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
    visible: bool,

    layout: GameplayWidgetLayout,
    original_layout: GameplayWidgetLayout,
    default_layout: GameplayWidgetLayout,

    hover: bool,
    selected: bool,
    bounds: Bounds,
}
impl WidgetState {
    fn new(widget: &GameplayWidgetContainer) -> Self {
        let layout = widget.layout().clone();

        Self {
            name: widget.element_name.clone(),
            visible: widget.visible,
            hover: false,
            selected: false,
            layout: layout.clone(),
            original_layout: layout,
            default_layout: widget.default_layout.clone(),

            bounds: widget.resolved_bounds(),
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        if self.hover {
            shell.list.push(graphics::Rectangle::new_bounds(
                self.bounds,
                Color::TRANSPARENT,
            ).border(Border::new(
                shell.general_theme.hover_color,
                2.0
            )));
        } else if self.selected {
            shell.list.push(graphics::Rectangle::new_bounds(
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
