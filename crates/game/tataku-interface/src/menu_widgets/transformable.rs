use crate::prelude::*;
use tataku::{
    Vector2,
    TatakuValue,
    Easing,
    Animate,
    AnimationTimeline,
};
use ui::{
    tree::*,
    widget::*,
    message::*,
};
use input::{ 
    InputEvent, 
    InputType 
};

#[derive(ChainableInitializer)]
pub struct TransformableWidget {
    child: Box<dyn Widget<actions::Action>>,

    x_position: AnimationTimeline<f32>,
    y_position: AnimationTimeline<f32>,
    x_scale: AnimationTimeline<f32>,
    y_scale: AnimationTimeline<f32>,
    rotation: AnimationTimeline<f32>,

    triggers: Vec<AnimatableTrigger>,
    actions: HashMap<String, Vec<AnimatableAction>>,

    node_id: NodeId,

    last_input: Option<f32>,
    hold_start: Option<f32>,

    skip_noinput_actions: Vec<AnimatableTriggerEvent>,
    skip_clickhold_actions: Vec<AnimatableTriggerEvent>,
}
impl TransformableWidget {
    pub fn new(
        triggers: Vec<AnimatableTrigger>,
        actions: HashMap<String, Vec<AnimatableAction>>,
        child: Box<dyn Widget<actions::Action>>,
    ) -> Self {
        Self {
            x_position: AnimationTimeline::new(Vec::new(), 0.0),
            y_position: AnimationTimeline::new(Vec::new(), 0.0),
            x_scale: AnimationTimeline::new(Vec::new(), 1.0),
            y_scale: AnimationTimeline::new(Vec::new(), 1.0),
            rotation: AnimationTimeline::new(Vec::new(), 0.0),

            triggers,
            actions,
            child,

            node_id: ui::EMPTY_NODE,
            last_input: None,
            hold_start: None,
            skip_clickhold_actions: Vec::new(),
            skip_noinput_actions: Vec::new(),
        }
    }

    pub fn with_animation(
        mut self,
        start_time: f32,
        duration: f32,
        easing: Easing,
        transform_type: TransformTypeTag,
    ) -> Self {
        self.push_animation(start_time, duration, easing, transform_type);
        self
    }

    pub fn push_animation(
        &mut self,
        start_time: f32,
        duration: f32,
        easing: Easing,
        transform_type: TransformTypeTag,
    ) {
        match transform_type {
            TransformTypeTag::Position { start, end } => {
                self.x_position.push(Animate::new(start_time, duration, easing, start.x, end.x));
                self.y_position.push(Animate::new(start_time, duration, easing, start.x, end.x));
            },
            TransformTypeTag::PositionX { start, end } =>
                self.x_position.push(Animate::new(start_time, duration, easing, start, end)),
            TransformTypeTag::PositionY { start, end } =>
                self.y_position.push(Animate::new(start_time, duration, easing, start, end)),

            TransformTypeTag::VectorScale { start, end } => {
                self.x_scale.push(Animate::new(start_time, duration, easing, start.x, end.x));
                self.y_scale.push(Animate::new(start_time, duration, easing, start.x, end.x));
            },
            TransformTypeTag::Scale { start, end } => {
                self.x_scale.push(Animate::new(start_time, duration, easing, start, end));
                self.y_scale.push(Animate::new(start_time, duration, easing, start, end));
            }
            TransformTypeTag::ScaleX { start, end } =>
                self.x_scale.push(Animate::new(start_time, duration, easing, start, end)),
            TransformTypeTag::ScaleY { start, end } =>
                self.y_scale.push(Animate::new(start_time, duration, easing, start, end)),
            TransformTypeTag::Rotation { start, end } =>
                self.rotation.push(Animate::new(start_time, duration, easing, start, end)),
            TransformTypeTag::None => {},
        }
    }

    fn run_triggers(&mut self, triggers: Vec<String>, time: f32) {
        for trigger in triggers {
            let Some(actions) = self.actions.get(&trigger).cloned()
            else { continue };

            for action in actions {
                let easing = Easing::Linear; // todo: get from action

                self.push_animation(time, action.duration, easing, action.action);
            }
        }
    }

    fn transform(&self) -> graphics::Transform {
        let x_position = self.x_position.last_value();
        let y_position = self.y_position.last_value();
        let x_scale = self.x_scale.last_value();
        let y_scale = self.y_scale.last_value();
        let rotation = self.rotation.last_value();

        graphics::Transform::new(
            Vector2::new(x_position, y_position),
            Vector2::new(x_scale, y_scale),
            rotation,
            Vector2::ZERO
        )
    }
}
impl Widget<actions::Action> for TransformableWidget {
    fn name(&self) -> CowStr { "transformable_widget".into() }
    fn node_id(&self) -> NodeId { self.node_id }

    fn children(&self) -> WidgetChildren<'_, actions::Action> {
        WidgetChildren::Single(&*self.child)
    }
    fn children_mut(&mut self) -> WidgetChildrenMut<'_, actions::Action> {
        WidgetChildrenMut::Single(&mut *self.child)
    }

    fn layout(&mut self, shell: &mut LayoutShell<actions::Action>) -> taffy::TaffyResult<NodeId>  {
        let child = self.child.layout(shell)?;
        self.node_id = shell.tree.new_with_children(&[child])?;
        Ok(self.node_id)
    }

    fn input(
        &mut self,
        event: &InputEvent,
        shell: &mut InputShell<actions::Action>,
    ) {
        let game_time = shell.values.reflect_get::<f32>("game.time")
            .unwrap()
            .copied();
        self.last_input = Some(game_time);
        self.skip_noinput_actions.clear();

        let Some(state) = shell.state_mut(self.node_id)
        else { return };

        let mut to_trigger = Vec::new();
        let mut clicked = false;
        let mut released = false;
        let mut hovered = false;
        let mut unhovered = false;

        match &event.event {
            InputType::MousePress(_mb) => if state.hover() {
                self.hold_start = Some(game_time);
                state.set_pressed(true);
                clicked = true;
            }
            InputType::MouseRelease(_mb) if state.pressed() => {
                self.hold_start = None;
                state.set_pressed(false);
                released = true;
                self.skip_clickhold_actions.clear();
            }
            InputType::MouseMove(pos) => {
                let bounds = shell
                    .tree
                    .content_bounds(self.node_id)
                    .unwrap();
                let ctx = shell
                    .tree
                    .get_context_mut(self.node_id)
                    .unwrap();

                let pos = ctx.inverse_global_transform * *pos;
                let new_hover = bounds.contains(pos);

                if new_hover != ctx.element_data.state.hover() {
                    ctx.element_data.state.set_hover(new_hover);
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

    fn update(&mut self, shell: &mut UpdateShell<actions::Action>) {
        let time = shell.values.reflect_get::<f32>("game.time")
            .unwrap()
            .copied();

        let mut to_trigger = Vec::new();
        for trigger in self.triggers.iter() {
            match &trigger.trigger {
                AnimatableTriggerEvent::NoInput { duration } => {
                    if let Some(last) = (self.last_input)
                        .filter(|_| self.hold_start.is_none())
                    && time - last >= *duration
                    && !self.skip_noinput_actions.contains(&trigger.trigger)
                    {
                        to_trigger.push(trigger.action.clone());
                        self.skip_noinput_actions.push(trigger.trigger.clone());
                    }
                }

                AnimatableTriggerEvent::ClickHold { duration } => {
                    if let Some(start) = self.hold_start
                    && time - start >= *duration
                        && !self.skip_clickhold_actions.contains(&trigger.trigger) {
                        to_trigger.push(trigger.action.clone());
                        self.skip_clickhold_actions.push(trigger.trigger.clone());
                    }
                }

                _ => {}
            }
        }
        self.run_triggers(to_trigger, time);

        let should_update = self.x_position.update(time)
            || self.y_position.update(time)
            || self.rotation.update(time)
            || self.x_scale.update(time)
            || self.y_scale.update(time);

        if should_update {
            let transform = self.transform();
            let context = shell
                .tree
                .get_context_mut(self.node_id)
                .unwrap();

            context.local_transform = transform;
            shell.tree.mark_dirty(self.node_id);
            // shell.actions.push(actions::ui::UiAction::new(
            //     self.node_id,
            //     shell.source,
            //     actions::ui::UiActionType::ContextChanged
            // ).into());
        }
        self.child.update(shell);
    }

    fn handle_message(
        &mut self,
        message: &Message,
        shell: &mut MessageShell<actions::Action>,
    ) {
        let mut to_trigger = Vec::new();

        for trigger in self.triggers.iter() {
            let AnimatableTriggerEvent::Message(tag) = &trigger.trigger
            else { continue };

            if &*message.tag == tag {
                to_trigger.push(trigger.action.clone());
            }
        }
        if !to_trigger.is_empty() {
            let time = shell.values
                .reflect_get::<f32>("game.time")
                .unwrap()
                .copied();
            self.run_triggers(to_trigger, time);
        }

        self.child.handle_message(message, shell);
    }

    fn handle_event(
        &mut self,
        event: &input::TatakuEvent,
        event_value: Option<&TatakuValue>,
        shell: &mut MessageShell<actions::Action>,
    ) {
        let mut to_trigger = Vec::new();
        for trigger in self.triggers.iter() {
            let AnimatableTriggerEvent::Event(trigger_event) = &trigger.trigger
            else { continue };
            if event == trigger_event {
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

}
