use crate::prelude::*;

#[derive(ChainableInitializer)]
pub struct Button {
    #[chain] on_press_left: ButtonOnClick,
    #[chain] on_press_middle: ButtonOnClick,
    #[chain] on_press_right: ButtonOnClick,
    child: Box<dyn Widget<TatakuAction>>,
    
    active_cond: VisuallyActive,

    /// did a click start on us (and the cursor has not moved)
    active: Option<MouseButton>,
    hovered: bool,

    node_id: NodeId,
}
impl Button {
    pub fn new(child: Box<dyn Widget<TatakuAction>>) -> Self {
        Self {
            child,
            node_id: EMPTY_NODE,
            on_press_left: ButtonOnClick::Message(None),
            on_press_middle: ButtonOnClick::Message(None),
            on_press_right: ButtonOnClick::Message(None),
            active_cond: VisuallyActive::None,

            active: None,
            hovered: false,
        }
    }

    pub fn active_condition(mut self, mut cond: BuildableCondition) -> Self {
        cond.build();
        self.active_cond = VisuallyActive::Condition { cond, value: false };
        self
    }

    pub fn active_condition_maybe(self, cond: Option<BuildableCondition>) -> Self {
        let Some(cond) = cond else { return self };
        self.active_condition(cond)
    }
    

    pub fn on_press(self, on_press: impl Into<ButtonOnClick>) -> Self {
        self.on_press_left(on_press)
    }
}
impl Widget<TatakuAction> for Button {
    fn name(&self) -> CowStr { "button_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, TatakuAction> {
        WidgetChildren::Single(&self.child)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, TatakuAction> {
        WidgetChildrenMut::Single(&mut self.child)
    }

    fn layout(&mut self, shell: &mut LayoutShell<TatakuAction>) -> taffy::TaffyResult<NodeId>  {
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
        shell: &mut InputShell<TatakuAction>,
    ) {
        let Some(bounds) = shell.tree.bounds(self.node_id) 
        else { return };

        let context = shell.tree.get_context(self.node_id).unwrap();
 
        match &event.event {
            InputType::MouseMove(pos) => {
                if self.active.is_some() { self.active = None }
                let pos = context.inverse_global_transform * *pos;
                self.hovered = bounds.contains(pos);
            }
            InputType::MouseScroll {..} if self.active.is_some() => self.active = None,
            
            InputType::MousePress(mb) if self.hovered => {
                self.active = Some(*mb);
                shell.event_consumed = true;
            }

            InputType::MouseRelease(mb) if self.active == Some(*mb) => {
                let action = match mb {
                    MouseButton::Left => &self.on_press_left,
                    MouseButton::Middle => &self.on_press_middle,
                    MouseButton::Right => &self.on_press_right,
                    _ => return,
                };

                if let Some(message) = action.resolve(
                    self.node_id,
                    None,
                    shell.values
                ) {
                    match message {
                        ActionResponse::Message(message) 
                            => shell.publish(message),

                        ActionResponse::Action(action) 
                            => shell.actions.push(action),
                    }
                    // shell.event_consumed = true;
                    return;
                }
            }

            _ => {}
        }

        self.child.input(event, shell);
    }
    
    fn draw(&self, shell: &mut DrawShell<TatakuAction>) {
        let theme = &shell.general_theme;
        let Some(bounds) = shell.tree.absolute_bounds(self.node_id) 
        else { return };

        let active = self.active.is_some() || self.active_cond.get();

        // draw button
        shell.list.push(
            Rectangle::new_bounds(
                bounds,
                theme.background_color,
            )
            .border(Border::new(
                theme.get_color(active, self.hovered), 
                2.0
            ))
            .shape(Shape::Round(2.0))
        );

        // draw child ontop of button
        self.child.draw(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell<TatakuAction>) {
        self.active_cond.update(shell.values);
        self.child.update(shell);

        let Some(ctx) = shell.tree.get_context_mut(self.node_id)
        else { return };

        let active = ctx.element_data.state.contains(ElementState::Active);
        let new_active = self.active_cond.get();
        if new_active != active {
            if new_active {
                ctx.element_data.state.insert(ElementState::Active);
            } else {
                ctx.element_data.state.remove(ElementState::Active);
            }
        }
    }

    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell<TatakuAction>,
    ) {
        self.child.handle_message(message, shell);
    }

}


type OnClickCallback = Box<dyn Fn() -> Option<Message> + Send + Sync>;

#[derive(Debug2)]
pub enum ButtonOnClick {
    Message(Option<Message>),
    BuildableActions(Vec<BuildableAction>),
    #[debug(skip)] Callback(OnClickCallback),
}
impl ButtonOnClick {
    pub fn resolve(
        &self, 
        node: NodeId,
        passed_in: Option<&TatakuValue>,
        values: &mut dyn Reflect,
    ) -> Option<ActionResponse> {
        match self {
            Self::Message(m) 
                => m.clone().map(ActionResponse::Message),

            Self::BuildableActions(actions) => {
                let actions = actions.iter().cloned()
                    .filter_map(|a| {
                        a.resolve(node, values, passed_in)
                    })
                    .collect::<Vec<_>>();

                if actions.is_empty() {
                    None
                } else {
                    Some(ActionResponse::Action(TatakuAction::Multiple(actions)))
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
impl<T: Into<ButtonOnClick>> From<Option<T>> for ButtonOnClick {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl From<Message> for ButtonOnClick {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
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
            if let BuildableAction::Conditional {
                cond,
                ..
            } = action {
                cond.build();
            }
        }

        if actions.is_empty() {
            Self::Message(None)
        } else {
            Self::BuildableActions(actions)
        }
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
            BuildableConditionResult::Unbuilt(_) => unreachable!("should be built"),
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


#[derive(Debug)]
pub enum ActionResponse {
    Message(Message),
    Action(TatakuAction),
}
