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
    /// This should never happpen, but its here to avoid unwraps
    NumberIsntANumber(String),
}
