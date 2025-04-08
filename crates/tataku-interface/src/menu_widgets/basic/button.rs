use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(ChainableInitializer)]
#[derive(Widget)]
#[widget(type("container"))]
pub struct Button {
    #[chain] pub style: Style,
    #[chain] pub on_press: ButtonOnClick,
    pub child: Box<dyn Widget>,
    
    visual_active_cond: VisuallyActive,

    /// did a click start on us (and the cursor has not moved)
    active: bool,
    hovered: bool,

    node_id: NodeId,
}
impl Button {
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            style: Style::DEFAULT,
            child,
            node_id: EMPTY_NODE,
            on_press: ButtonOnClick::Message(None),
            visual_active_cond: VisuallyActive::None,

            hovered: false,
            active: false,
        }
    }

    pub fn active_condition(mut self, mut cond: BuildableCondition) -> Self {
        cond.build();
        self.visual_active_cond = VisuallyActive::Condition { cond, value: false };
        self
    }

    pub fn active_condition_maybe(self, cond: Option<BuildableCondition>) -> Self {
        let Some(cond) = cond else { return self };
        self.active_condition(cond)
    }
    
}

#[async_trait]
impl Widget for Button {
    fn name(&self) -> Cow<'static, str> { "button_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(&mut self, tree: &mut Tree, resolver: &mut CssResolver, display_override: Option<ui::Display>) {
        self.child.update_styles(tree, resolver, display_override);
    }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId>  {
        let child = self.child.layout(shell)?;
        self.node_id = shell.tree.new_with_children(self.style.clone(), &[ child ])?;
        
        shell.with_context(self.node_id, |ctx| {
            ctx.needs_inverse_transform = true;
            ctx.set_selectable(true);
        });
        
        Ok(self.node_id)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell,
    ) {
        let Some(bounds) = shell.tree.bounds(self.node_id) else { return };
        let context = shell.tree.get_context(self.node_id).unwrap();
 
        match &event.event {
            InputType::MouseMove(pos) => {
                if self.active { self.active = false }
                let pos = context.inverse_global_transform * *pos;
                self.hovered = bounds.contains(pos);
            }
            InputType::MouseScroll(_) if self.active => self.active = false,
            
            InputType::MousePress(MouseButton::Left) if self.hovered => {
                self.active = true;
                shell.event_consumed = true;
            }

            InputType::MouseRelease(MouseButton::Left) if self.active => {
                if let Some(message) = self.on_press.resolve(
                    shell.owner,
                    None,
                    shell.values
                ) {
                    match message {
                        ActionResponse::Message(message) => shell.publish(message),
                        ActionResponse::Action(action) => shell.actions.push(action),
                    }
                    // shell.event_consumed = true;
                    return;
                }
            }

            _ => {}
        }

        self.child.input(event, shell);
    }
    
    
    fn draw(
        &self, 
        shell: &mut DrawShell<'_>,
    ) {
        let theme = &shell.general_theme;
        let Some(bounds) = shell.tree.absolute_bounds(self) else { return };

        let active = self.active || self.visual_active_cond.get();

        // draw button
        shell.list.push(Rectangle::new_bounds(
            bounds,
            theme.background_color,
            Some(Border::new(theme.get_color(active, self.hovered), 2.0)),
        ).shape(Shape::Round(2.0)));

        // draw child ontop of button
        self.child.draw(shell);
    }
    
    fn update(
        &mut self, 
        shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        self.visual_active_cond.update(shell.values);
        self.child.update(shell, actions);
    }

    async fn handle_message(
        &mut self, 
        message: &Message, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        self.child.handle_message(message, values, actions).await;
    }

    async fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        values: &mut dyn Reflect
    ) {
        self.child.handle_event(event, event_value, values).await
    }

    async fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.child.reload_skin(shell).await
    }
}


type OnClickCallback = Box<dyn Fn() -> Option<Message> + Send + Sync>;

#[derive(Debug2)]
pub enum ButtonOnClick {
    Message(Option<Message>),
    BuildableAction(BuildableAction),
    #[debug(skip)] 
    Callback(OnClickCallback),
    // ActionCallback(OnClickaActionCallback),
}
impl ButtonOnClick {
    pub fn resolve(
        &self, 
        _owner: MessageOwner,
        passed_in: Option<TatakuValue>,
        values: &mut dyn Reflect
    ) -> Option<ActionResponse> {
        match self {
            Self::Message(m) => m.clone().map(ActionResponse::Message),
            Self::BuildableAction(action) => action.clone().into_action(values, passed_in).map(ActionResponse::Action),
            Self::Callback(cb) => (cb)().map(ActionResponse::Message),
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
    fn from(mut value: BuildableAction) -> Self {
        if let BuildableAction::Conditional { cond, .. } = &mut value {
            cond.build();
        }

        Self::BuildableAction(value)
    }
}
impl From<OnClickCallback> for ButtonOnClick {
    fn from(value: OnClickCallback) -> Self {
        Self::Callback(value)
    }
}
impl From<ButtonBuilderOnClick> for ButtonOnClick {
    fn from(value: ButtonBuilderOnClick) -> Self {
        match value {
            ButtonBuilderOnClick::Message(message) => Self::Message(message),
            ButtonBuilderOnClick::Callback(cb) => Self::Callback(cb),
        }
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
        let Self::Condition { cond, value } = self else { return };
        match cond.resolve(values) {
            BuildableConditionResult::Failed => {},
            BuildableConditionResult::Unbuilt(_) => unreachable!("should be built"),
            BuildableConditionResult::True => *value = true,
            BuildableConditionResult::False => *value = false,
            BuildableConditionResult::Error(shunting_yard_error) => {
                error!("Error with shunting yeard: {shunting_yard_error:?}");
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
