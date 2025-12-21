use crate::prelude::*;
use tataku::{
    Border,
    Bounds,
    Vector2,
};
use ui::{
    tree::*,
    style::*,
    widget::*,
};
use input::{ 
    Key,
    InputType,
    InputEvent, 
    MouseButton, 
};
use widgets::context_menus::ContextMenuAction;

/// NOTE: This doesn't need to be in the taffy tree (probably)
pub struct ContextMenu {
    pub options: Vec<ContextMenuOption>,
    pub hover_index: Option<usize>,
    pub active_index: Option<usize>,

    /// should this context menu be closed?
    pub should_close: bool,

    /// should this context menu's parent also be closed?
    pub should_close_parent: bool,

    /// top-left
    pub location: Vector2,
    text_style: TextStyle,
    item_size: Vector2,

    submenu: Option<Box<Self>>,

    node_id: NodeId,
}
impl ContextMenu {
    pub fn new(
        options: Vec<ContextMenuOption>,
        location: Vector2,
    ) -> Self {
        let text_style = TextStyle {
            ..Default::default()
        };

        // let item_size = options
        //     .iter()
        //     .map(|o| text_style.measure_text(
        //         &o.name,
        //         None
        //     ))
        //     .fold(
        //         Vector2::ZERO,
        //         |i, n| Vector2::new(
        //             i.x.max(n.x),
        //             i.y.max(n.y)
        //         )
        //     ) + Vector2::new(5.0, 5.0);

        let item_size = Vector2::new(5.0, 5.0);

        Self {
            options,
            hover_index: None,
            active_index: None,
            submenu: None,
            should_close: false,
            should_close_parent: false,

            location,
            text_style,
            item_size,
            node_id: NodeId::default(),
        }
    }

    fn index_at(&self, y: f32) -> Option<usize> {
        let len = self.options.len() as f32;
        let rel_y = y - self.location.y;

        let index = (rel_y / self.item_size.y).floor();

        if index < 0.0 || index >= len { 
            None
        } else {
            Some(index as usize)
        }
    }

    fn try_make_submenu(&mut self, index: usize) -> bool {
        let option = &self.options[index];
        let ContextMenuOptionType::SubMenu(menu) 
            = &option.option_type
        else { return false };

        let location = Vector2::new(
            self.item_size.x,
            self.item_size.y * index as f32,
        );

        let mut submenu = menu();
        submenu.location = self.location + location;

        self.submenu = Some(Box::new(submenu));

        true
    }
}
impl Widget<actions::Action> for ContextMenu {
    fn name(&self) -> CowStr { "context_menu".into() }
    fn node_id(&self) -> &NodeId { &self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }

    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        shell.tree.update_style(
            &self.node_id, 
            |s| s.position = Position::Absolute.into()
        );
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<actions::Action>,
    ) {
        if let Some(menu) = &mut self.submenu {
            menu.input(event, shell);
        }

        if shell.event_consumed { return }

        match &event.event {
            InputType::MouseMove(pos) => {
                let bounds = Bounds::new(
                    self.location,
                    Vector2::new(
                        self.item_size.x,
                        self.item_size.y * self.options.len() as f32
                    )
                );
                if !bounds.contains(*pos) {
                    return;
                }

                self.hover_index = self.index_at(pos.y);
                if self.active_index.is_some() 
                    && self.active_index != self.hover_index 
                {
                    self.active_index = None;
                    self.submenu = None;
                }

                if let Some(hover) = self.hover_index
                && !self.try_make_submenu(hover) {
                    self.submenu = None;
                }
            }

            InputType::MousePress(MouseButton::Left) => {
                self.active_index = self.index_at(shell.mouse_pos.y);
                if self.active_index.is_some() {
                    shell.event_consumed = true;
                }
            }

            InputType::MouseRelease(MouseButton::Left) => {
                if let Some(active) = self.active_index {
                    shell.event_consumed = true;
                    if let ContextMenuOptionType::Action(action) 
                        = &self.options[active].option_type
                    {
                        self.should_close = true;
                        self.should_close_parent = true;
                        action.run(
                            shell.tree.node.node_id(),
                            None,
                            shell.values,
                            shell.actions,
                            shell.messages,
                        );
                    }
                }
            }

            InputType::KeyPress(key) => {
                if key.is_key(Key::Up) {
                    if let Some(index) = &mut self.active_index {
                        *index = (*index - 1).clamp(0, self.options.len());
                    }
                } else if key.is_key(Key::Down) || key.is_key(Key::Tab) {
                    if let Some(index) = &mut self.active_index {
                        *index = (*index + 1).clamp(0, self.options.len());
                    } else {
                        self.active_index = Some(0);
                    }
                } else if key.is_key(Key::Enter) || key.is_key(Key::Space) {
                    if let Some(active) = self.active_index {
                        match &self.options[active].option_type {
                            ContextMenuOptionType::TextOnly => {}
                            ContextMenuOptionType::SubMenu(_) => {
                                self.try_make_submenu(active);
                            }

                            ContextMenuOptionType::Action(action) => {
                                self.should_close = true;
                                self.should_close_parent = true;
                                action.run(
                                    shell.tree.node.node_id(),
                                    None,
                                    shell.values,
                                    shell.actions,
                                    shell.messages
                                );
                            }
                        }
                    }
                } 

                // TODO: need to vary between left and right depending on which way the context menu opens
                // ie if the menu opens to the left (non-default) these will need to be reversed
                else if key.is_key(Key::Left) {
                    if self.active_index.is_some() {
                        self.should_close = true;
                        self.should_close_parent = false;
                    }
                } else if key.is_key(Key::Right) { 
                    if let Some(active) = self.active_index {
                        self.try_make_submenu(active);

                        // if a submenu was created, init it with the active index 0 (first), 
                        // so that its automatically ready for keyboard controls
                        if let Some(menu) = &mut self.submenu {
                            menu.active_index = Some(0);
                        }
                    }
                } 
                
                else {
                    return;
                }
                shell.event_consumed = true;
            }

            _ => {}
        }
    }

    #[allow(clippy::only_used_in_recursion, reason = "required")]
    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        if let Some(menu) = &mut self.submenu {
            menu.update(shell);
            if menu.should_close {
                if menu.should_close_parent {
                    self.should_close = true;
                    self.should_close_parent = true;
                }

                self.submenu = None;
            }
        }
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        for (n, i) in self.options.iter().enumerate() {
            let pos = self.location 
                + Vector2::new(0.0, self.item_size.y * n as f32);
            let bounds = Bounds::new(pos, self.item_size);

            shell.list.push(graphics::Rectangle::new_bounds(
                bounds,
                shell.general_theme.background_color,
            ).border(Border::new(
                shell.general_theme.get_color(
                    self.active_index == Some(n), 
                    self.hover_index == Some(n)
                ),
                2.0
            )));

            // shell.list.push(self.text_style.create_text(
            //     i.name.clone().into_owned(),
            //     bounds
            // ));
        }

        if let Some(menu) = &self.submenu {
            menu.draw(shell);
        }
    }
}

pub struct ContextMenuOption {
    pub name: CowStr,
    pub option_type: ContextMenuOptionType,
}
impl ContextMenuOption {
    pub fn new(
        name: impl Into<CowStr>,
        option_type: impl Into<ContextMenuOptionType>,
    ) -> Self {
        Self {
            name: name.into(),
            option_type: option_type.into(),
        }
    }
}

pub type ContextMenuBuilder = Box<dyn Fn() -> ContextMenu + Send + Sync>;

#[derive(From)]
pub enum ContextMenuOptionType {
    SubMenu(ContextMenuBuilder),
    Action(Box<ContextMenuAction>),
    TextOnly,
}
impl From<ContextMenuAction> for ContextMenuOptionType {
    fn from(value: ContextMenuAction) -> Self {
        Self::Action(Box::new(value))
    }
}
