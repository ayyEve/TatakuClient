use crate::*;
use ::shunting_yard::TokenType;

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f32),
    Variable(VariablePathResolver),
    StringLiteral(String),
    Operation(crate::Operator),
    Function(String, usize),
    OpenParenthesis,
}
impl<'values> ::shunting_yard::Token<
    'values, 
    Cow<'values, tataku::TatakuValue>, 
    Error
> for Token {
    type Operator = crate::Operator;
    const OPEN_PAREN: Self = Self::OpenParenthesis;

    fn get_type(&self) -> TokenType {
        match self {
            Self::Number(_) 
            | Self::Variable(_) 
            | Self::StringLiteral(_) 
                => TokenType::Value,

            Self::Operation(_) => TokenType::Operator,
            Self::Function(_,_) => TokenType::Function,
            Self::OpenParenthesis => TokenType::Unknown,
        }
    }

    fn as_operator(&self) -> Option<&Self::Operator> {
        match self {
            Self::Operation(o) => Some(o),
            _ => None,
        }
    }

    fn from_operator(op: Self::Operator) -> Self {
        Self::Operation(op)
    }

    fn set_arg_count(&mut self, count: usize) {
        let Self::Function(_, arg_count) = self 
        else { panic!("trying to set arg count for non-function token") };

        *arg_count = count;
    }
}
