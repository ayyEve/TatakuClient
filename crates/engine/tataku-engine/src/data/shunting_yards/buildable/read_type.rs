use crate::prelude::*;

/// helper for parsing numbers and variables
#[doc(hidden)]
#[derive(Default)]
pub enum BuildableShuntingYardReadType {
    #[default] None,
    Number(String),
    StringLiteral(String),
    Variable(String),
}
impl BuildableShuntingYardReadType {
    pub(super) fn push(&mut self, c: char) {
        match self {
            Self::None => match c {
                '0'..='9' => *self = Self::Number(format!("{c}")),
                'a'..='z'|'.'|'_'|':' => *self = Self::Variable(format!("{c}")),
                _ => {}
            }
            Self::Number(s) => s.push(c),
            Self::Variable(s) => s.push(c),
            Self::StringLiteral(s) => s.push(c),
        }
    }
}
impl _ShuntingYardReadType for BuildableShuntingYardReadType {}
