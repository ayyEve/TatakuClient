use crate::prelude::*;
use tataku::Border;
use common::reflect::*;
use widgets::InputAction;
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

pub trait InputButtonType: Send + Sync {
    type Output: Copy + std::fmt::Debug + Reflect + PartialEq + Send + Sync;
    fn press_txt() -> CowStr;

    fn from_event(e: &InputEvent) -> Option<Self::Output>;
}
impl InputButtonType for input::Key {
    type Output = Self;
    fn press_txt() -> CowStr { "Press a Key".into() }

    fn from_event(e: &InputEvent) -> Option<Self::Output> {
        let InputType::KeyPress(key) = &e.event
        else { return None };

        let Some(key) = key.as_key() else {
            error!("couldnt convert KeyInput to Key: {key:?}");
            return None;
        };

        Some(key)
    }
}
impl InputButtonType for input::GamepadButton {
    type Output = Self;
    fn press_txt() -> CowStr { "Press a Controller Button".into() }
    
    fn from_event(e: &InputEvent) -> Option<Self::Output> {
        let InputType::ControllerPress(btn, _, _) = &e.event
        else { return None };

        Some(*btn)
    }
}


#[derive(ChainableInitializer)]
pub struct InputButton<T: InputButtonType> {
    input: InputButtonValue<T::Output>,
    #[chain] optional: bool,
    on_change: Option<InputAction<Option<T::Output>>>,

    node_id: NodeId,
}
impl<T: InputButtonType> InputButton<T> {
    pub fn new(input: InputButtonValue<T::Output>) -> Self {
        Self {
            input,
            optional: false,
            
            on_change: None,
            node_id: ui::EMPTY_NODE,
        }
    }

    pub fn on_change(mut self, on_change: Option<impl Into<InputAction<Option<T::Output>>>>) -> Self {
        self.on_change = on_change.map(Into::into);
        self
    }

    fn text(&self, active: bool) -> CowStr {
        if active {
            T::press_txt()
        } else if let Some(i) = self.input.get() {
            format!("{i:?}").into()
        } else {
            "None".into()
        }
    }
}
impl<T: InputButtonType> Widget<actions::Action> for InputButton<T> {
    fn name(&self) -> CowStr { "input_button".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId> {
        self.node_id = shell.tree.new_leaf()?;
        Ok(self.node_id)
    }
    fn init_style(&mut self, shell: &mut LayoutShell<actions::Action>) {
        let text_style = shell.tree
            .get_text_style(self.node_id)
            .unwrap();

        // let w = f16::from_f32(text_style.measure_text("Press a key", None).x);
        let h = f16::from_f32(text_style.line_height);
        shell.tree.update_style(
            self.node_id, 
            |style| {
                // style.min_width = CssUnit::Pixels(w).into();
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
        shell: &mut InputShell<actions::Action>,
    ) {
        if shell.event_consumed { return }
        let bounds = shell.tree
            .absolute_bounds(self.node_id)
            .unwrap();

        let ctx = shell.tree.get_context_mut(self.node_id).unwrap();
        if ctx.element_data.state.contains(ElementState::Active) && event.is_keyboard()
        && let InputType::KeyPress(key) = &event.event {
            ctx.element_data.state.remove(ElementState::Active);
            shell.event_consumed = true;

            if key.is_key(Key::Escape) 
            && event.key_mods.ctrl && self.optional 
            {
                if let InputButtonValue::Static(k) = &mut self.input {
                    *k = None;
                }

                if let Some(on_change) = &self.on_change {
                    on_change.run(
                        &None,
                        self.node_id,
                        shell.source,
                        shell.messages,
                        shell.actions,
                        shell.values,
                    );
                }
            }
        }

        if !shell.event_consumed 
        && let Some(i) = T::from_event(event)
        {
            if let InputButtonValue::Static(k) = &mut self.input {
                *k = Some(i);
            }

            if let Some(on_change) = &self.on_change {
                on_change.run(
                    &Some(i),
                    self.node_id,
                    shell.source,
                    shell.messages,
                    shell.actions,
                    shell.values,
                );
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


    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        if self.input.update(shell.values, self.optional) {
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

            shell.tree.mark_dirty(self.node_id);
        }
    }


    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let ctx = shell.tree
            .get_context(self.node_id)
            .unwrap();

        let active = ctx.element_data.state.contains(ElementState::Active);
        let hover = ctx.element_data.state.contains(ElementState::Hover);

        let bounds = shell.tree
            .absolute_bounds(self.node_id)
            .unwrap();

        shell.list.push(graphics::Rectangle::new_bounds(
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


pub enum InputButtonValue<T> {
    Static(Option<T>),
    Variable {
        path: engine::VariablePathResolver,
        cache: Option<T>, 
        error_logged: bool,
    },
}
impl<T:Copy + Reflect + PartialEq> InputButtonValue<T> {
    pub(crate) fn get(&self) -> Option<T> {
        match self {
            Self::Static(k) 
            | Self::Variable { cache: k, .. }
                => *k,
        }
    }

    pub(crate) fn update(
        &mut self, 
        values: &dyn Reflect,
        optional: bool,
    ) -> bool {
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
        if optional {
            if let Some(&input) = val.downcast_ref::<Option<T>>()
            && *cache != input {
                *cache = input;
                return true;
            }
        } else if let Some(&input) = val.downcast_ref::<T>()
        && *cache != Some(input) {
            *cache = Some(input);
            return true;
        }

        false
    }
}
impl<T> From<engine::VariablePathResolver> for InputButtonValue<T> {
    fn from(path: engine::VariablePathResolver) -> Self {
        Self::Variable {
            path,
            cache: None,
            error_logged: false,
        }
    }
}
