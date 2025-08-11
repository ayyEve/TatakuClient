use crate::prelude::*;
use super::*;

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PathShuntingYardOperator;
impl _ShuntingYardOperator<'_> for PathShuntingYardOperator {
    type Output = String;
    type Error = PathShuntingYardError;

    fn precedence(&self) -> u8 { 0 }
    fn is_left_associative(&self) -> bool { false }
    fn single_arg(&self) -> bool { false }

    fn read(c1: char, c2: char) -> Result<Self, _ShuntingYardOperatorReadError> {
        match (c1, c2) {
            (':', ':') => Ok(Self),
            _ => Err(_ShuntingYardOperatorReadError::Ignore)
        }
    }

    fn perform(
        &self, 
        _: Self::Output, 
        _: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error> {
        unimplemented!()
    }
}
