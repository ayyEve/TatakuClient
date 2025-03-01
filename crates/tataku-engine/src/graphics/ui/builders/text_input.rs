use crate::prelude::*;

#[derive(Default)]
pub struct TextInputBuilder {
    pub placeholder: TextBuilderValue,
    pub value: TextBuilderValue,
    pub font_size: Option<f32>,
    pub on_input: TextInputBuilderInput,
    pub on_submit: TextInputBuilderInput,
    pub secure: bool,
}
impl TextInputBuilder {
    pub fn new(placeholder: impl Into<TextBuilderValue>, value: impl Into<TextBuilderValue>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: value.into(),
            ..Default::default()
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
    
    pub fn on_input(mut self, on_input: impl Into<TextInputBuilderInput>) -> Self {
        self.on_input = on_input.into();
        self
    }
    pub fn on_submit(mut self, on_submit: impl Into<TextInputBuilderInput>) -> Self {
        self.on_submit = on_submit.into();
        self
    }
}


type TextInputCallback = Box<dyn Fn(&str) -> Message + Send + Sync>;

pub enum TextInputBuilderInput {
    Message(Option<Message>),
    Callback(TextInputCallback),
}
impl Default for TextInputBuilderInput {
    fn default() -> Self { Self::Message(None) }
}
impl<T: Into<TextInputBuilderInput>> From<Option<T>> for TextInputBuilderInput {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl From<Message> for TextInputBuilderInput {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl From<TextInputCallback> for TextInputBuilderInput {
    fn from(value: TextInputCallback) -> Self {
        Self::Callback(value)
    }
}
