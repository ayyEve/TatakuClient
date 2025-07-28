use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct KeyButton {
    pub key: KeyButtonValue,
    #[chain] pub optional: bool,
    #[chain] pub on_change: InputAction<Option<Key>>,

    node_id: NodeId,
}
impl KeyButton {
    pub fn new(
        key: impl Into<KeyButtonValue>, 
    ) -> Self {
        Self {
            key: key.into(),
            optional: false,
            
            on_change: InputAction::default(),
            node_id: NodeId::default(),
        }
    }

    fn text(&self, active: bool) -> CowStr {
        if active {
            "Press a key".into()
        } else if let Some(k) = self.key.get() {
            format!("{k:?}").into()
        } else {
            "None".into()
        }
    }
}
impl Widget for KeyButton {
    fn name(&self) -> CowStr { "key_input".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId> {
        let mut text_style = TextStyle::default();
        text_style.font_size *= shell.ui_scale;

        let style = Style {
            min_size: Size {
                width: Dimension::Length(text_style.measure_text("Press a key", None).x),
                height: Dimension::Length(text_style.line_height),
            },

            ..Style::default()
        };

        self.node_id = shell.tree.new_leaf(style)?;
        Ok(self.node_id)
    }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell,
        _display_override: Option<ui::Display>
    ) {
        let text_size = shell.tree
            .get_context(self.node_id)
            .unwrap()
            .element_data.style()
            .0.text_style(shell.values)
            .measure_text("Press a key", None)
            ;

        let mut style = shell.tree.get_style(self.node_id).unwrap().clone();
        style.min_size = Size {
            width: Dimension::Length(text_size.x),
            height: Dimension::Length(text_size.y),
        };
        shell.tree.set_style(self.node_id, style);
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell,
    ) {
        if !self.on_change.is_built() {
            self.on_change.build(shell.values);
        }

        if shell.event_consumed { return }
        let bounds = shell.tree
            .absolute_bounds(self.node_id)
            .unwrap();

        let ctx = shell.tree.get_context_mut(self.node_id).unwrap();
        if ctx.element_data.state.contains(ElementState::Active) && event.is_keyboard() {
            if let InputType::KeyPress(key) = &event.event {
                ctx.element_data.state.remove(ElementState::Active);
                shell.event_consumed = true;

                if key.is_key(Key::Escape) {
                    if event.key_mods.ctrl && self.optional {
                        if let KeyButtonValue::Static(k) = &mut self.key {
                            *k = None;
                        }

                        self.on_change.run(
                            &None,
                            self.node_id,
                            shell.messages,
                            shell.actions,
                            shell.values,
                        );
                    }
                } else {
                    let Some(key) = key.as_key() else {
                        error!("couldnt convert KeyInput to Key: {key:?}");
                        return;
                    };
                    if let KeyButtonValue::Static(k) = &mut self.key {
                        *k = Some(key);
                    }

                    self.on_change.run(
                        &Some(key),
                        self.node_id,
                        shell.messages,
                        shell.actions,
                        shell.values,
                    );
                }
            }
        }
        let hover = ctx.element_data.state.contains(ElementState::Hover);

        match event.event {
            InputType::MouseMove(pos) => {
                if bounds.contains(pos) {
                    ctx.element_data.state.insert(ElementState::Hover);
                } else if hover {
                    ctx.element_data.state.remove(ElementState::Hover);
                }
            }

            InputType::MousePress(MouseButton::Left) => {
                if hover {
                    ctx.element_data.state.insert(ElementState::Active);
                } else if ctx.element_data.state.contains(ElementState::Active) {
                    ctx.element_data.state.remove(ElementState::Active);
                }
            }

            _ => {}
        }
        
    }


    fn update(&mut self, shell: &mut UpdateShell) {
        if self.key.update(shell.values) {
            let ctx = shell
                .tree
                .get_context(self.node_id)
                .unwrap();

            let txt = ctx
                .element_data
                .style().0
                .text_style(shell.values);

            let active = ctx.element_data
                .state
                .contains(ElementState::Active);

            let min = txt.measure_text(&self.text(active), None);

            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::UpdateStyleWith(Box::new(
                    move |style| style.min_size = min.into()
                ))
            ));
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::MarkDirty,
            ));
        }
    }


    fn draw(&self, shell: &mut DrawShell) {
        let ctx = shell.tree
            .get_context(self.node_id)
            .unwrap();

        let active = ctx.element_data.state.contains(ElementState::Active);
        let hover = ctx.element_data.state.contains(ElementState::Hover);

        let bounds = shell.tree
            .absolute_bounds(self.node_id)
            .unwrap();

        shell.list.push(Rectangle::new_bounds(
            bounds, 
            shell.general_theme.background_color,
        ).border(Border::new(
            shell.general_theme.get_color(active, hover), 
            1.2
        )));

        let style = ctx
            .element_data
            .style().0
            .text_style(shell.values);

        shell.list.push(style.create_text(
            self.text(active).into_owned(), 
            bounds
        ));
    }
}


pub enum KeyButtonValue {
    Static(Option<Key>),
    Variable(String, Option<Key>),
}
impl KeyButtonValue {
    fn get(&self) -> Option<Key> {
        match self {
            Self::Static(k) 
            | Self::Variable(_, k)
                => *k,
        }
    }

    fn update(&mut self, values: &dyn Reflect) -> bool {
        let Self::Variable(path, cache) = self 
        else { return false };

        let Ok(val) = values.impl_get(ReflectPath::new(path)) 
        else { return false };

        let val = val.as_ref();
        if let Some(&key) = val.downcast_ref::<Option<Key>>() {
            if *cache != key {
                *cache = key;
                true
            } else {
                false
            }
        } else if let Some(&key) = val.downcast_ref::<Key>() {
            if *cache != Some(key) {
                *cache = Some(key);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
impl From<KeyButtonBuilderValue> for KeyButtonValue {
    fn from(value: KeyButtonBuilderValue) -> Self {
        match value {
            KeyButtonBuilderValue::Static(k) => Self::Static(k),
            KeyButtonBuilderValue::Variable(path) => Self::Variable(path, None),
        }
    }
}

impl From<KeyButtonBuilderInput> for InputAction<Option<Key>> {
    fn from(value: KeyButtonBuilderInput) -> Self {
        match value {
            KeyButtonBuilderInput::Callback(cb)
                => Self::MessageCallback(cb),
            KeyButtonBuilderInput::Message(m) => Self::Message(m),
        }
    }
}
