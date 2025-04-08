use crate::prelude::*;

pub struct ButtonBuilder {
    pub element: Box<dyn Widget>,
    pub on_press: ButtonBuilderOnClick
}
impl ButtonBuilder {
    pub fn new(element: Box<dyn Widget>) -> Self {
        Self {
            element,
            on_press: Default::default(),
        }
    }

    pub fn on_press(mut self, m: impl Into<ButtonBuilderOnClick>) -> Self {
        self.on_press = m.into();
        self
    }
}


type OnClickCallback = Box<dyn Fn() -> Option<Message> + Send + Sync>;
pub enum ButtonBuilderOnClick {
    Message(Option<Message>),
    Callback(OnClickCallback),
}
impl Default for ButtonBuilderOnClick {
    fn default() -> Self {
        Self::Message(None)
    }
}
impl<T: Into<ButtonBuilderOnClick>> From<Option<T>> for ButtonBuilderOnClick {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl From<Message> for ButtonBuilderOnClick {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl From<OnClickCallback> for ButtonBuilderOnClick {
    fn from(value: OnClickCallback) -> Self {
        Self::Callback(value)
    }
}
