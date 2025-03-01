use crate::prelude::*;
use std::ops::RangeInclusive;

pub struct SliderBuilder {
    pub range: RangeInclusive<f32>,
    pub value: SliderBuilderValue,
    pub on_change: SliderBuilderOnChange,
    pub step: Option<f32>,
}
impl SliderBuilder {
    pub fn new(range: RangeInclusive<f32>, value: impl Into<SliderBuilderValue>) -> Self {
        Self {
            range,
            value: value.into(),
            on_change: SliderBuilderOnChange::Message(None),
            step: None,
        }
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    
    pub fn on_change(mut self, on_change: impl Into<SliderBuilderOnChange>) -> Self {
        self.on_change = on_change.into();
        self
    }
}


pub enum SliderBuilderValue {
    Static(f32),
    Variable(String),
}
impl From<f32> for SliderBuilderValue {
    fn from(value: f32) -> Self {
        Self::Static(value)
    }
}
impl From<&str> for SliderBuilderValue {
    fn from(value: &str) -> Self {
        Self::Variable(value.to_owned())
    }
}
impl From<String> for SliderBuilderValue {
    fn from(value: String) -> Self {
        Self::Variable(value.to_string())
    }
}


type OnChangeCallback = Box<dyn Fn(f32) -> Message + Send + Sync>;
pub enum SliderBuilderOnChange {
    Message(Option<Message>),
    Callback(OnChangeCallback),
}
impl From<Message> for SliderBuilderOnChange {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl From<OnChangeCallback> for SliderBuilderOnChange {
    fn from(value: OnChangeCallback) -> Self {
        Self::Callback(value)
    }
}
