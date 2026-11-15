
pub trait Operator<'values>: Sized {
    type Output;
    type Error;

    fn read(c1: char, c2: char) -> Result<Self, OperatorReadError>;
    fn precedence(&self) -> u8;
    fn is_left_associative(&self) -> bool;

    fn single_arg(&self) -> bool { false }

    fn perform(
        &self, 
        a: Self::Output, 
        b: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error>;
}

#[derive(Copy, Clone, Debug)]
pub enum OperatorReadError {
    Unknown,
    Ignore,
}
