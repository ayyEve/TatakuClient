use super::*;
use shunting_yard::TokenType;

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Static(String),
    Operation(Operator),
    Reference,
    OpenParenthesis,
}
impl shunting_yard::Token<'_, String, Error> for Token {
    type Operator = crate::Operator;
    const OPEN_PAREN: Self = Self::OpenParenthesis;

    fn get_type(&self) -> TokenType {
        match self {
            Self::Static(_) => TokenType::Value,
            Self::Reference => TokenType::Function,
            Self::Operation(_) => TokenType::Operator,
            Self::OpenParenthesis => TokenType::Unknown,
        }
    }

    fn as_operator(&self) -> Option<&Self::Operator> {
        let Self::Operation(op) = self 
        else { return None };

        Some(op)
    }

    fn from_operator(op: Self::Operator) -> Self {
        Self::Operation(op)
    }

    fn set_arg_count(&mut self, _count: usize) {}
}
