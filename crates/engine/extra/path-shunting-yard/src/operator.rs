use shunting_yard::OperatorReadError;

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Operator;
impl shunting_yard::Operator<'_> for Operator {
    type Output = String;
    type Error = crate::Error;

    fn precedence(&self) -> u8 { 0 }
    fn is_left_associative(&self) -> bool { false }
    fn single_arg(&self) -> bool { false }

    fn read(c1: char, c2: char) -> Result<Self, OperatorReadError> {
        match (c1, c2) {
            (':', ':') => Ok(Self),
            _ => Err(OperatorReadError::Ignore)
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
