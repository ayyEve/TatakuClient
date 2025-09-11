use crate::*;

#[doc(hidden)]
#[derive(Default)]
pub(crate) enum PathShuntingYardReadType {
    #[default] None,
    Static(String),
}
impl PathShuntingYardReadType {
    pub fn push(&mut self, c: char) {
        match self {
            Self::Static(s) => s.push(c),
            Self::None => *self = Self::Static(c.to_string()),
        }
    }
}
impl tataku::_ShuntingYardReadType for PathShuntingYardReadType {}
