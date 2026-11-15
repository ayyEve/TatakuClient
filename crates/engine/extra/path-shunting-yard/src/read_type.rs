
#[doc(hidden)]
#[derive(Default)]
pub(crate) enum ReadType {
    #[default] None,
    Static(String),
}
impl ReadType {
    pub fn push(&mut self, c: char) {
        match self {
            Self::Static(s) => s.push(c),
            Self::None => *self = Self::Static(c.to_string()),
        }
    }
}
// impl shunting_yard::ReadType for PathShuntingYardReadType {}
