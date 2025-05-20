use crate::prelude::*;
use crate::prelude::ui::*;

#[derive(Widget)]
#[widget(type("container"))]
#[derive(ChainableInitializer)]
pub struct TransformableWidget {
    #[chain] style: Style,
    child: Box<dyn Widget>,
    
    /// we let a TransformGroup handle the transforms to avoid duplicating code
    manager: TransformManager,
    triggers: Vec<AnimatableTrigger>,
    actions: HashMap<String, Vec<AnimatableAction>>,

    node_id: NodeId,

    // FIXME: pressed and hold_start technically do the same thing
    hover: bool,
    pressed: bool,
    last_input: Option<f32>,
    hold_start: Option<f32>,

    skip_noinput_actions: Vec<AnimatableTriggerEvent>,
    skip_clickhold_actions: Vec<AnimatableTriggerEvent>,
}
impl TransformableWidget {
    pub fn new(
        triggers: Vec<AnimatableTrigger>,
        actions: HashMap<String, Vec<AnimatableAction>>,
        child: Box<dyn Widget>,
    ) -> Self {
        Self {
            style: Style::default(),

            manager: TransformManager::new(Vector2::ZERO),
            triggers,
            actions,
            child,

            node_id: EMPTY_NODE,

            hover: false,
            pressed: false,
            last_input: None,
            hold_start: None,
            skip_clickhold_actions: Vec::new(),
            skip_noinput_actions: Vec::new(),
        }
    }

    pub fn add_transform(&mut self, transform: Transformation) {
        self.manager.push_transform(transform);
    }
    pub fn with_transform(mut self, transform: Transformation) -> Self {
        self.add_transform(transform);
        self
    }


    fn run_triggers(&mut self, triggers: Vec<String>, time: f32) {
        for trigger in triggers {
            let Some(actions) = self.actions.get(&trigger) else { continue };
            for action in actions {
                self.manager.push_transform(Transformation::new(
                    0.0,
                    action.duration,
                    action.action.into(),
                    // TODO: action.easing,
                    Easing::Linear,
                    time
                ));


            }
        }
    }
}
impl Widget for TransformableWidget {
    fn name(&self) -> Cow<'static, str> { "transformable_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn update_styles(
        &mut self, 
        shell: &mut StyleShell, 
        display_override: Option<ui::Display>
    ) {
        self.child.update_styles(shell, display_override);
    }
    
    fn layout(&mut self, shell: &mut LayoutShell) -> TaffyResult<NodeId>  {
        let child = self.child.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            self.style.clone(),
            &[child]
        )?;

        Ok(self.node_id)
    }

    fn input(
        &mut self, 
        event: &InputEvent, 
        shell: &mut InputShell,
    ) {
        let game_time = shell.values.reflect_get::<f32>("game.time").unwrap().copied();
        self.last_input = Some(game_time);
        self.skip_noinput_actions.clear();

        let mut to_trigger = Vec::new();
        let mut clicked = false;
        let mut released = false;
        let mut hovered = false;
        let mut unhovered = false;

        match &event.event {
            InputType::MousePress(_mouse_button) => if self.hover {
                self.hold_start = Some(game_time);
                self.pressed = true;
                clicked = true;
            }
            InputType::MouseRelease(_mouse_button) if self.pressed => {
                self.hold_start = None;
                self.pressed = false;
                released = true;
                self.skip_clickhold_actions.clear();
            }
            InputType::MouseMove(pos) => {
                let bounds = shell.tree.content_bounds(self.node_id).unwrap();
                let ctx = shell.tree.get_context(self.node_id).unwrap();

                let pos = ctx.inverse_global_transform * *pos;
                let new_hover = bounds.contains(pos);

                if new_hover != self.hover {
                    self.hover = new_hover;
                    if new_hover {
                        hovered = true;
                    } else {
                        unhovered = true;
                    }
                }
            }

            _ => {}
        }

        for trigger in self.triggers.iter() {
            match trigger.trigger {
                AnimatableTriggerEvent::Input => to_trigger.push(trigger.action.clone()),
                AnimatableTriggerEvent::Hover if hovered => to_trigger.push(trigger.action.clone()),
                AnimatableTriggerEvent::Unhover if unhovered => to_trigger.push(trigger.action.clone()),
                AnimatableTriggerEvent::Click if clicked => to_trigger.push(trigger.action.clone()),
                AnimatableTriggerEvent::Unclick if released => to_trigger.push(trigger.action.clone()),
                _ => {}
            }
        }

        self.run_triggers(to_trigger, game_time);

        self.child.input(event, shell);
    }

    fn draw(&self, shell: &mut DrawShell) {
        self.child.draw(shell);
    }
    fn draw_overlay(&self, shell: &mut DrawShell) {
        self.child.draw_overlay(shell);
    }

    fn update(&mut self, shell: &mut UpdateShell) {
        let time = shell.values.reflect_get::<f32>("game.time")
            .unwrap()
            .copied();

        let mut to_trigger = Vec::new();
        for trigger in self.triggers.iter() {
            match &trigger.trigger {
                AnimatableTriggerEvent::NoInput { duration } => {
                    if let Some(last) = (self.last_input).filter(|_| self.hold_start.is_none() ) {
                        if time - last >= *duration && !self.skip_noinput_actions.contains(&trigger.trigger) {
                            to_trigger.push(trigger.action.clone());
                            self.skip_noinput_actions.push(trigger.trigger.clone());
                        }
                    }
                }

                AnimatableTriggerEvent::ClickHold { duration } => {
                    if let Some(start) = self.hold_start {
                        if time - start >= *duration && !self.skip_clickhold_actions.contains(&trigger.trigger) {
                            to_trigger.push(trigger.action.clone());
                            self.skip_clickhold_actions.push(trigger.trigger.clone());
                        }
                    }
                }

                _ => {}
            }
        }
        self.run_triggers(to_trigger, time);

        let should_update = !self.manager.transforms.is_empty();
        self.manager.update(time);
        if should_update {
            let context = shell.tree.get_context_mut(self.node_id).unwrap();
            context.local_transform = Transform::from_manager(&self.manager);
            shell.actions.push(UiAction::new(
                self.node_id, 
                UiActionType::ContextChanged
            ));
        }
        self.child.update(shell);
    }
    
    fn handle_message(
        &mut self, 
        message: &Message, 
        shell: &mut MessageShell,
    ) {
        let mut to_trigger = Vec::new();

        for trigger in self.triggers.iter() {
            let AnimatableTriggerEvent::Message(tag) = &trigger.trigger 
            else { continue };

            if &message.tag == tag {
                to_trigger.push(trigger.action.clone());
            } 
        }
        if !to_trigger.is_empty() {
            let time = shell.values.reflect_get::<f32>("game.time").unwrap().copied();
            self.run_triggers(to_trigger, time);
        }

        self.child.handle_message(message, shell);
    }

    fn handle_event(
        &mut self, 
        event: TatakuEventType, 
        event_value: Option<TatakuValue>, 
        shell: &mut MessageShell,
    ) {
        let mut to_trigger = Vec::new();
        for trigger in self.triggers.iter() {
            let AnimatableTriggerEvent::Event(trigger_event) = &trigger.trigger else { continue };
            if &event == trigger_event {
                to_trigger.push(trigger.action.clone());
            }
        }
        if !to_trigger.is_empty() {
            let time = shell.values.reflect_get::<f32>("game.time")
                .unwrap().copied();
            self.run_triggers(to_trigger, time);
        }
        
        self.child.handle_event(event, event_value, shell);
    }

    fn reload_skin(&mut self, shell: &mut UpdateShell) {
        self.child.reload_skin(shell);
    }
}
