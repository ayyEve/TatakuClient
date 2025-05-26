use crate::prelude::*;

#[derive(Default)]
#[derive(ChainableInitializer)]
pub struct KeyButtonBuilder {
    pub value: KeyButtonBuilderValue,
    #[chain] pub on_change: KeyButtonBuilderInput,
    #[chain] pub optional: bool,
}
impl KeyButtonBuilder {
    pub fn new(
        value: impl Into<KeyButtonBuilderValue>
    ) -> Self {
        Self {
            value: value.into(),
            ..Default::default()
        }
    }
}


pub enum KeyButtonBuilderValue {
    Static(Option<Key>),
    Variable(String),
}
impl From<Option<Key>> for KeyButtonBuilderValue {
    fn from(value: Option<Key>) -> Self {
        Self::Static(value)
    }
}
impl From<Key> for KeyButtonBuilderValue {
    fn from(value: Key) -> Self {
        Self::Static(Some(value))
    }
}
impl From<String> for KeyButtonBuilderValue {
    fn from(value: String) -> Self {
        Self::Variable(value)
    }
}

impl Default for KeyButtonBuilderValue {
    fn default() -> Self {
        Self::Static(None)
    }
}

type KeyButtonCallback = Box<dyn Fn(&Option<Key>) -> Message + Send + Sync>;

pub enum KeyButtonBuilderInput {
    Message(Option<Message>),
    Callback(KeyButtonCallback),
}
impl Default for KeyButtonBuilderInput {
    fn default() -> Self { Self::Message(None) }
}
impl<T: Into<KeyButtonBuilderInput>> From<Option<T>> for KeyButtonBuilderInput {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl From<Message> for KeyButtonBuilderInput {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl From<KeyButtonCallback> for KeyButtonBuilderInput {
    fn from(value: KeyButtonCallback) -> Self {
        Self::Callback(value)
    }
}
