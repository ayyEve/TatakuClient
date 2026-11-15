use tataku_engine_common::common::common::reflect::ReflectError;

#[doc(hidden)]
#[derive(Debug, PartialEq)]
pub enum Error {
    Unknown,
    EmptyExpression,

    UnhandledToken(crate::Token),
    ReflectError(ReflectError<'static>)
}
impl shunting_yard::Error for Error {
    type Operator = crate::Operator;
    type Token = crate::Token;

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
impl<'a> From<ReflectError<'a>> for Error {
    fn from(value: ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}
