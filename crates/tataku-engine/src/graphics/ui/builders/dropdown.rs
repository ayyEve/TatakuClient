use crate::prelude::*;

pub struct DropdownBuilder {
    pub variants: DropdownBuilderVariants,
    pub value: DropdownBuilderValue,
    pub on_change: DropdownBuilderOnChange,
    pub font_size: Option<f32>,
}
impl DropdownBuilder {
    pub fn new(
        variants: impl Into<DropdownBuilderVariants>, 
        value: impl Into<DropdownBuilderValue>,
    ) -> Self {
        Self {
            variants: variants.into(),
            value: value.into(),
            on_change: DropdownBuilderOnChange::default(),
            font_size: None,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }
    
    pub fn on_change(mut self, on_change: impl Into<DropdownBuilderOnChange>) -> Self {
        self.on_change = on_change.into();
        self
    }
}


pub enum DropdownBuilderVariants {
    Static(Vec<String>),
    Variable(String),
}
impl From<Vec<String>> for DropdownBuilderVariants {
    fn from(value: Vec<String>) -> Self {
        Self::Static(value)
    }
}
impl From<String> for DropdownBuilderVariants {
    fn from(value: String) -> Self {
        Self::Variable(value)
    }
}


pub enum DropdownBuilderValue {
    Index(Option<usize>),
    Variable(String),
}
impl From<usize> for DropdownBuilderValue {
    fn from(value: usize) -> Self {
        Self::Index(Some(value))
    }
}
impl From<Option<usize>> for DropdownBuilderValue {
    fn from(value: Option<usize>) -> Self {
        Self::Index(value)
    }
}
impl From<String> for DropdownBuilderValue {
    fn from(value: String) -> Self {
        Self::Variable(value)
    }
}


type OnChange = Box<dyn Fn(usize) -> Message + Send + Sync>;
pub enum DropdownBuilderOnChange {
    Message(Option<Message>),
    Callback(OnChange),
}
impl Default for DropdownBuilderOnChange {
    fn default() -> Self {
        Self::Message(None)
    }
}
impl<T: Into<DropdownBuilderOnChange>> From<Option<T>> for DropdownBuilderOnChange {
    fn from(value: Option<T>) -> Self {
        let Some(value) = value else { return Self::Message(None) };
        value.into()
    }
}
impl From<Message> for DropdownBuilderOnChange {
    fn from(value: Message) -> Self {
        Self::Message(Some(value))
    }
}
impl From<OnChange> for DropdownBuilderOnChange {
    fn from(value: OnChange) -> Self {
        Self::Callback(value)
    }
}
