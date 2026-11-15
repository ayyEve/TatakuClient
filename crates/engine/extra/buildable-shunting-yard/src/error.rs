use crate::*;
use common::reflect::ReflectError;

pub type ShuntingYardResult<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    NoMath,
    UnexpectedComma,
    InvalidOperator(char),
    InvalidToken(crate::Token),
    InvalidFunction(String),
    EntryDoesntExist(String),
    ValueIsntANumber(String),
    ValueIsntABool,
    MissingLeftSide(crate::Operator),
    MissingRightSide(crate::Operator),
    MissingFunctionArgument(String),

    ArgumentWrongCount {
        function: String,
        expected: usize,
        received: usize,
    },
    ArgumentWrongType {
        expected: String,
        received: String,
    },
    NumberIsntANumber(String),

    ValueIsNone,

    ConversionError(String),
    InvalidType(String),

    ReflectError(ReflectError<'static>)
}

impl<'a> From<ReflectError<'a>> for Error {
    fn from(value: ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}

impl ::shunting_yard::Error for Error {
    type Operator = crate::Operator;
    type Token = crate::Token;

    const NO_OPERATION: Self = Self::NoMath;
    const UNEXPECTED_COMMA: Self = Self::UnexpectedComma;

    fn unhandled_token(token: &Self::Token) -> Self {
        Self::InvalidToken(token.clone())
    }
    fn missing_left_side(op: &Self::Operator) -> Self {
        Self::MissingLeftSide(*op)
    }
    fn missing_right_side(op: &Self::Operator) -> Self {
        Self::MissingRightSide(*op)
    }

    fn wrong_argument_count(
        function: String, 
        expected: usize, 
        received: usize,
    ) -> Self {
        Self::ArgumentWrongCount { function, expected, received }
    }
}

