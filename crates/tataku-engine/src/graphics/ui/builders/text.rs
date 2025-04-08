pub struct TextBuilder {
    pub text: TextBuilderValue,
    pub font_size: Option<f32>,
}
impl TextBuilder {
    pub fn new(text: impl Into<TextBuilderValue>) -> Self {
        Self {
            text: text.into(),
            font_size: None
        }
    }

    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = Some(font_size);
        self
    }
}


pub enum TextBuilderValue {
    Static(String),
    Variable(String),
    Calc(String),
    List(Vec<Self>, String)
}
impl Default for TextBuilderValue {
    fn default() -> Self {
        Self::Static(String::new())
    }
}
impl From<&String> for TextBuilderValue {
    fn from(value: &String) -> Self {
        Self::Static(value.to_owned())
    }
}
impl From<&str> for TextBuilderValue {
    fn from(value: &str) -> Self {
        Self::Static(value.to_owned())
    }
}
impl From<String> for TextBuilderValue {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}