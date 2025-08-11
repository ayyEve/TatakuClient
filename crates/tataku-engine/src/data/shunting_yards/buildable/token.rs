use crate::prelude::*;

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum BuildableShuntingYardToken {
    Number(f32),
    Variable(String),
    StringLiteral(String),
    Operation(BuildableShuntingYardOperator),
    Function(String, usize),
    OpenParenthesis,
}
impl<'values> _ShuntingYardToken<
    'values, 
    Cow<'values, TatakuValue>, 
    BuildableShuntingYardError
> for BuildableShuntingYardToken {
    type Operator = BuildableShuntingYardOperator;
    const OPEN_PAREN: Self = Self::OpenParenthesis;

    fn get_type(&self) -> _ShuntingYardTokenType {
        match self {
            Self::Number(_) 
            | Self::Variable(_) 
            | Self::StringLiteral(_) 
                => _ShuntingYardTokenType::Value,

            Self::Operation(_) => _ShuntingYardTokenType::Operator,
            Self::Function(_,_) => _ShuntingYardTokenType::Function,
            Self::OpenParenthesis => _ShuntingYardTokenType::Unknown,
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
