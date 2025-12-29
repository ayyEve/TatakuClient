use crate::*;

pub trait Token<'values, Output, Error>: PartialEq {
    type Operator: Operator<'values, Output = Output, Error = Error>;
    const OPEN_PAREN: Self;

    fn get_type(&self) -> TokenType;
    fn as_operator(&self) -> Option<&Self::Operator>;
    fn from_operator(op: Self::Operator) -> Self;
    fn set_arg_count(&mut self, count: usize);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    Value,
    Function,
    Operator,
    Unknown,
}
