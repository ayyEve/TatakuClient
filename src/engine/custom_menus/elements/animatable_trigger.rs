use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct AnimatableTrigger {
    pub trigger: AnimatableTriggerEvent,
    pub action: String,
}

#[derive(Clone, Debug)]
pub enum AnimatableTriggerEvent {
    Input,
    NoInput { duration: f32 },
    Hover,
    Unhover,
    Click,
    ClickHold { duration: f32 }, 
    Unclick,
}

#[derive(Clone, Debug)]
pub struct AnimatableAction {
    pub action: TransformType,
    pub start: AnimatableTransformValue,
    pub stop: AnimatableTransformValue,
    pub duration: f32
}

#[derive(Clone, Debug)]
pub enum AnimatableTransformValue {
    Val(f32),
    Current,
    ParentWidth,
    ParentHeight,
}
