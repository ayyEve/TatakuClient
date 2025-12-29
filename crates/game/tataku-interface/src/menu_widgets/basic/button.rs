use crate::prelude::*;
use common::reflect::*;
use tataku::TatakuValue;
use ui::{
    tree::*,
    widget::*,
    message::*,
};
use input::{ 
    InputType,
    InputEvent, 
    MouseButton, 
};

#[derive(ChainableInitializer)]
pub struct Button<T = Box<dyn Widget<actions::Action>>> {
    #[chain] pub on_press_left: Option<ButtonOnClick>,
    #[chain] pub on_press_middle: Option<ButtonOnClick>,
    #[chain] pub on_press_right: Option<ButtonOnClick>,
    pub child: T,
    
    #[chain] active_cond: VisuallyActive,

    /// did a click start on us (and the cursor has not moved)
    pressed: Option<MouseButton>,

    node_id: NodeId,
}
impl<T> Button<T> {
    pub fn new(child: T) -> Self {
        Self {
            child,
            node_id: ui::EMPTY_NODE,
            on_press_left: None,
            on_press_middle: None,
            on_press_right: None,
            active_cond: VisuallyActive::None,

            pressed: None,
        }
    }

    pub fn into_widget_base(self) -> widgets::WidgetBase<Self> {
        widgets::WidgetBase::new(
            ArcStr::default(),
            "button".into(),
            None,
            ClassList::default(),
            self,
        )
    }
}
impl<T> Widget<actions::Action> for Button<T>
where
    T: Widget<actions::Action>
{
    fn name(&self) -> CowStr { "button_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::Single(&self.child)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::Single(&mut self.child)
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId>  {
        let child = self.child.layout(shell)?;

        self.node_id = shell.tree.new_with_children(&[ child ])?;
        
        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });
        
        Ok(self.node_id)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell<actions::Action>,
    ) {
        let Some(bounds) = shell.tree.bounds(self.node_id) 
        else { return };

        let Some(ctx) = shell.tree.get_context_mut(self.node_id)
        else { return };
        let state = &mut ctx.element_data.state;
 
        match &event.event {
            InputType::MouseMove(pos) => {
                if self.pressed.is_some() { 
                    self.pressed = None;
                    state.set_active(false);
                }
                let pos = ctx.inverse_global_transform * *pos;
                state.set_hover(bounds.contains(pos));
            }
            InputType::MouseScroll {..} if self.pressed.is_some() => {
                state.set_active(false);
                self.pressed = None;
            }
            
            InputType::MousePress(mb) if state.hover() => {
                self.pressed = Some(*mb);
                state.set_active(true);
                shell.event_consumed = true;
            }
            InputType::MousePressCancel(mb) => {
                if self.pressed == Some(*mb) {
                    self.pressed = None;
                    state.set_active(false);
                    shell.event_consumed = true;
                }
            }

            InputType::MouseRelease(mb) if self.pressed == Some(*mb) => {
                let action = match mb {
                    MouseButton::Left => &self.on_press_left,
                    MouseButton::Middle => &self.on_press_middle,
                    MouseButton::Right => &self.on_press_right,
                    _ => return,
                };
                
                let Some(action) = action else { return; };

                if let Some(message) = action.resolve(
                    self.node_id,
                    shell.source,
                    None,
                    shell.values
                ) {
                    match message {
                        ActionResponse::Message(message) 
                            => shell.messages.push(message),

                        ActionResponse::Action(action) 
                            => shell.actions.push(action),
                    }
                    shell.event_consumed = true;
                    return;
                }
            }

            _ => {}
        }

        self.child.input(event, shell);
    }
    
    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        self.active_cond.update(shell.values);
        self.child.update(shell);

        let Some(state) = shell.state_mut(self.node_id)
        else { return };

        state.set_active(self.active_cond.get());
    }

    fn draw(&self, shell: &mut DrawShell<actions::Action>) {
        let theme = &shell.general_theme;
        
        let Some((bounds, state)) = shell.with_ctx(
            self.node_id, 
            |ctx| (ctx.absolute_bounds, ctx.element_data.state)
        ) else { return };

        // draw button
        shell.list.push(
            graphics::Rectangle::new_bounds(
                bounds,
                theme.background_color,
            ).border(tataku::Border::new(
                theme.for_state(state), 
                2.0
            )).shape(graphics::Shape::Round(2.0))
        );

        // draw child ontop of button
        self.child.draw(shell);
    }


    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell<actions::Action>,
    ) {
        self.child.handle_message(message, shell);
    }

}


type OnClickCallback = Box<dyn Fn() -> Option<Message> + Send + Sync>;

#[derive(Debug2)]
pub enum ButtonOnClick {
    BuildableActions(Vec<BuildableAction>),
    #[debug(skip)] Callback(OnClickCallback),
}
impl ButtonOnClick {
    pub fn resolve(
        &self, 
        node: NodeId,
        source: ui::MessageSource,
        passed_in: Option<&TatakuValue>,
        values: &mut dyn Reflect,
    ) -> Option<ActionResponse> {
        match self {
            Self::BuildableActions(actions) => {
                let actions = actions.iter().cloned()
                    .filter_map(|a| {
                        a.resolve(node, source, values, passed_in)
                    })
                    .collect::<Vec<_>>();

                if actions.is_empty() {
                    None
                } else {
                    Some(ActionResponse::Action(actions::Action::Multiple(actions)))
                }
            },

            Self::Callback(cb) 
                => (cb)().map(ActionResponse::Message),
        }
    }

    pub fn build(&mut self) {
        let Self::BuildableActions(actions) = self 
        else { return };

        for a in actions {
            a.build();
        }
    }
}

impl From<BuildableAction> for ButtonOnClick {
    fn from(action: BuildableAction) -> Self {
        vec![action].into()
    }
}
impl From<Vec<BuildableAction>> for ButtonOnClick {
    fn from(mut actions: Vec<BuildableAction>) -> Self {
        for action in actions.iter_mut() {
            action.build();
            // if let BuildableAction::Conditional {
            //     cond,
            //     ..
            // } = action {
            //     cond.build();
            // }
        }

        Self::BuildableActions(actions)
    }
}

impl<F> From<Box<F>> for ButtonOnClick
where
    F: Fn() -> Option<Message> + Send + Sync + 'static
{
    fn from(value: Box<F>) -> Self {
        Self::Callback(value)
    }
}
impl From<OnClickCallback> for ButtonOnClick {
    fn from(value: OnClickCallback) -> Self {
        Self::Callback(value)
    }
}

// TODO: rename? 
enum VisuallyActive {
    None,
    Condition {
        cond: BuildableCondition,
        value: bool,
    }
}
impl VisuallyActive {
    fn update(&mut self, values: &dyn Reflect) {
        let Self::Condition { 
            cond, 
            value 
        } = self else { return };
        
        match cond.resolve(values) {
            BuildableConditionResult::Failed => {},
            BuildableConditionResult::Unbuilt(_) => panic!("should be built"),
            BuildableConditionResult::True => *value = true,
            BuildableConditionResult::False => *value = false,
            BuildableConditionResult::Error(shunting_yard_error) => {
                error!("Error with shunting yard: {shunting_yard_error:?}");
                *cond = BuildableCondition::Failed;
            }
        }
    } 
    fn get(&self) -> bool {
        match self {
            Self::None => false,
            Self::Condition { value, .. } => *value,
        }
    }
}
impl From<Option<BuildableCondition>> for VisuallyActive {
    fn from(value: Option<BuildableCondition>) -> Self {
        match value {
            None => Self::None,
            Some(mut cond) => {
                cond.build();
                Self::Condition { 
                    cond, 
                    value: false
                }
            }
        }
    }
}


#[derive(Debug)]
pub enum ActionResponse {
    Message(Message),
    Action(actions::Action),
}
