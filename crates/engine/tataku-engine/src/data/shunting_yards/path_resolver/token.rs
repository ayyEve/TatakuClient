use crate::prelude::*;
use super::*;

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum PathShuntingYardToken {
    Static(String),
    Operation(PathShuntingYardOperator),
    Reference,
    OpenParenthesis,
}
impl _ShuntingYardToken<'_, String, PathShuntingYardError> for PathShuntingYardToken {
    type Operator = PathShuntingYardOperator;
    const OPEN_PAREN: Self = Self::OpenParenthesis;

    fn get_type(&self) -> _ShuntingYardTokenType {
        match self {
            Self::Static(_) => _ShuntingYardTokenType::Value,
            Self::Reference => _ShuntingYardTokenType::Function,
            Self::Operation(_) => _ShuntingYardTokenType::Operator,
            Self::OpenParenthesis => _ShuntingYardTokenType::Unknown,
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
