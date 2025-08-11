use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
pub struct KeyButton {
    key: KeyButtonValue,
    #[chain] optional: bool,
    #[chain] on_change: InputAction<Option<Key>>,

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
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell) {
        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        let w = half::f16::from_f32(text_style.measure_text("Press a key", None).x);
        let h = half::f16::from_f32(text_style.line_height);
        shell.tree.update_style(
            self.node_id, 
            |style| {
                style.min_width = CssUnit::Pixels(w).into();
                style.min_height = CssUnit::Pixels(h).into();
            }
        );
    }

    // fn update_styles(
    //     &mut self, 
    //     shell: &mut StyleShell,
    //     _display_override: Option<ui::DisplayType>
    // ) {
    //     let text_size = shell.tree
    //         .get_context(self.node_id)
    //         .unwrap()
    //         .element_data.style()
    //         .0.text_style(shell.values)
    //         .measure_text("Press a key", None)
    //         ;

    //     shell.tree.update_style(self.node_id, |style| {
    //         style.min_width = CssUnit::Pixels(half::f16::from_f32(text_size.x)).into();
    //         style.min_height = CssUnit::Pixels(half::f16::from_f32(text_size.y)).into();
    //     });
    // }

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
                UiActionType::UpdateStyleWith(Arc::new(
                    move |style| {
                        style.min_width = CssUnit::Pixels(half::f16::from_f32(min.x)).into();
                        style.min_height = CssUnit::Pixels(half::f16::from_f32(min.y)).into();
                    }
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
    Variable {
        path: VariablePathResolver,
        cache: Option<Key>, 
        error_logged: bool,
    },
}
impl KeyButtonValue {
    fn get(&self) -> Option<Key> {
        match self {
            Self::Static(k) 
            | Self::Variable { cache: k, .. }
                => *k,
        }
    }

    fn update(&mut self, values: &dyn Reflect) -> bool {
        let Self::Variable {
            path, 
            cache, 
            error_logged
        } = self 
        else { return false };

        let Ok(path) = path
            .resolve_path(values) 
            .map_err(|e| {
                if !*error_logged {
                    error!("Error resolving path: {e:?}");
                    *error_logged = true;
                }
            })
        else { return false };

        let Ok(val) = values.impl_get(ReflectPath::new(&path)) 
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


impl From<VariablePathResolver> for KeyButtonValue {
    fn from(path: VariablePathResolver) -> Self {
        Self::Variable {
            path,
            cache: None,
            error_logged: false,
        }
    }
}
