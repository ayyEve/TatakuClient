use crate::prelude::*;
use tataku_engine::prelude::GamepadButton as GamepadButton;

#[derive(ChainableInitializer)]
pub struct GamepadButtonInput {
    button: InputButtonValue<GamepadButton>,
    #[chain] optional: bool,
    #[chain] on_change: InputAction<Option<GamepadButton>>,

    node_id: NodeId,
}
impl GamepadButtonInput {
    pub fn new(
        button: impl Into<InputButtonValue<GamepadButton>>, 
    ) -> Self {
        Self {
            button: button.into(),
            optional: false,
            
            on_change: InputAction::default(),
            node_id: NodeId::default(),
        }
    }

    fn text(&self, active: bool) -> CowStr {
        if active {
            "Press a Button".into()
        } else if let Some(b) = self.button.get() {
            format!("{b:?}").into()
        } else {
            "None".into()
        }
    }
}
impl Widget<TatakuAction> for GamepadButtonInput {
    fn name(&self) -> CowStr { "gamepad_input".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(
        &mut self, 
        shell: &mut LayoutShell<TatakuAction>
    ) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<TatakuAction>) {
        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        // let w = f16::from_f32(text_style.measure_text("Press a button", None).x);
        // let h = f16::from_f32(text_style.line_height);
        // shell.tree.update_style(
        //     self.node_id,
        //     |style| {
        //         style.min_width = CssUnit::Pixels(w).into();
        //         style.min_height = CssUnit::Pixels(h).into();
        //     }
        // );
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<TatakuAction>,
    ) {
        if !self.on_change.is_built() {
            self.on_change.build(shell.values);
        }

        if shell.event_consumed { return }
        let bounds = shell.tree
            .absolute_bounds(self.node_id)
            .unwrap();

        let ctx = shell
            .tree
            .get_context_mut(self.node_id)
            .unwrap();
        let state = &mut ctx.element_data.state;
        let active = state.contains(ElementState::Active);
        let hover = state.contains(ElementState::Hover);

        match &event.event {
            InputType::KeyPress(key) 
            if active && key.is_key(Key::Escape) => {
                state.remove(ElementState::Active);
                shell.event_consumed = true;

                if event.key_mods.ctrl && self.optional {
                    if let InputButtonValue::Static(
                        b
                    ) = &mut self.button {
                        *b = None;
                    }

                    self.on_change.run(
                        &None,
                        self.node_id,
                        shell.messages,
                        shell.actions,
                        shell.values,
                    );
                }
            }

            InputType::MouseMove(pos) => {
                if bounds.contains(*pos) {
                    state.insert(ElementState::Hover);
                } else if hover {
                    state.remove(ElementState::Hover);
                }
            }

            InputType::MousePress(MouseButton::Left) => {
                if hover {
                    state.insert(ElementState::Active);
                } else if active {
                    state.remove(ElementState::Active);
                }
            }

            InputType::ControllerPress(btn, _, _) if active => {
                state.remove(ElementState::Active);
                shell.event_consumed = true;
                
                if let InputButtonValue::Static(
                    b
                ) = &mut self.button {
                    *b = Some(*btn);
                }

                self.on_change.run(
                    &Some(*btn),
                    self.node_id,
                    shell.messages,
                    shell.actions,
                    shell.values,
                );
            }

            _ => {}
        }
        
    }


    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        if self.button.update(shell.values, self.optional) {
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

            // let min = txt.measure_text(&self.text(active), None);

            // shell.actions.push(UiAction::new(
            //     self.node_id,
            //     UiActionType::UpdateStyleWith(Arc::new(
            //         move |style| {
            //             style.min_width = CssUnit::Pixels(half::f16::from_f32(min.x)).into();
            //             style.min_height = CssUnit::Pixels(half::f16::from_f32(min.y)).into();
            //         }
            //     ))
            // ));
            // shell.actions.push(UiAction::new(
            //     self.node_id,
            //     UiActionType::MarkDirty,
            // ));
        }
    }


    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
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

        // shell.list.push(style.create_text(
        //     self.text(active).into_owned(),
        //     bounds
        // ));
    }
}
