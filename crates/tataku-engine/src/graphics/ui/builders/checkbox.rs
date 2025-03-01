use crate::prelude::*;

#[derive(Default)]
pub struct CheckboxBuilder {
    pub text: String,
    pub value: CheckboxBuilderValue,
    pub font_size: Option<f32>,

    pub on_change: Option<Arc<dyn Fn(bool) -> Message + Send + Sync>>,
}
impl CheckboxBuilder {
    pub fn new(text: impl ToString, value: impl Into<CheckboxBuilderValue>) -> Self {
        Self {
            text: text.to_string(),
            value: value.into(),
            ..Default::default()
        }
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }
    
    pub fn on_change(mut self, on_change: impl Fn(bool) -> Message + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(on_change));
        self
    }
}


pub enum CheckboxBuilderValue {
    Static(bool),
    Variable(String),
}
impl Default for CheckboxBuilderValue {
    fn default() -> Self {
        Self::Static(false)
    }
}
impl From<bool> for CheckboxBuilderValue {
    fn from(value: bool) -> Self {
        Self::Static(value)
    }
}
impl From<String> for CheckboxBuilderValue {
    fn from(value: String) -> Self {
        Self::Variable(value)
    }
}
