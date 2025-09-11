use crate::*;
use common::reflect::ReflectError;
use engine::data::shunting_yards::path_resolver::*;

#[doc(hidden)]
#[derive(Debug, PartialEq)]
pub enum PathShuntingYardError {
    Unknown,
    EmptyExpression,

    UnhandledToken(PathShuntingYardToken),
    ReflectError(ReflectError<'static>)
}
impl tataku::_ShuntingYardError for PathShuntingYardError {
    type Operator = PathShuntingYardOperator;
    type Token = PathShuntingYardToken;

    const NO_OPERATION: Self = Self::EmptyExpression;
    const UNEXPECTED_COMMA: Self = Self::Unknown;

    fn missing_left_side(_op: &Self::Operator) -> Self {
        Self::Unknown
    }

    fn missing_right_side(_op: &Self::Operator) -> Self {
        Self::Unknown
    }

    fn unhandled_token(token: &Self::Token) -> Self {
        Self::UnhandledToken(token.clone())
    }

    fn wrong_argument_count(
        _function: String, 
        _expected: usize, 
        _received: usize,
    ) -> Self {
        Self::Unknown
    }
}
impl<'a> From<ReflectError<'a>> for PathShuntingYardError {
    fn from(value: ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}
