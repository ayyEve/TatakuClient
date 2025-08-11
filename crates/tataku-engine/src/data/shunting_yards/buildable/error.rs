use crate::prelude::*;

pub type ShuntingYardResult<T> = Result<T, BuildableShuntingYardError>;

#[derive(Debug, PartialEq)]
pub enum BuildableShuntingYardError {
    NoMath,
    UnexpectedComma,
    InvalidOperator(char),
    InvalidToken(BuildableShuntingYardToken),
    InvalidFunction(String),
    EntryDoesntExist(String),
    ValueIsntANumber(String),
    ValueIsntABool,
    MissingLeftSide(BuildableShuntingYardOperator),
    MissingRightSide(BuildableShuntingYardOperator),
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

impl<'a> From<ReflectError<'a>> for BuildableShuntingYardError {
    fn from(value: ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}

impl _ShuntingYardError for BuildableShuntingYardError {
    type Operator = BuildableShuntingYardOperator;
    type Token = BuildableShuntingYardToken;

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

