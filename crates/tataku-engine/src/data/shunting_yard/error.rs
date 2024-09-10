use crate::prelude::*;

pub type ShuntingYardResult<T> = Result<T, ShuntingYardError>;

#[derive(Debug)]
pub enum ShuntingYardError {
    NoMath,
    InvalidOperator(char),
    InvalidToken(ShuntingYardToken),
    InvalidFunction(String),
    EntryDoesntExist(String),
    ValueIsntANumber(String),
    ValueIsntABool,
    MissingLeftSide(SYOperator),
    MissingRightSide(SYOperator),
    MissingFunctionArgument(String),
    NumberIsntANumber(String),

    ValueIsNone,

    ConversionError(String),
    InvalidType(String),

    ReflectError(ReflectError<'static>)
}

impl<'a> From<ReflectError<'a>> for ShuntingYardError {
    fn from(value: ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}