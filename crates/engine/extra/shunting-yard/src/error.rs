use tataku_common::reflect::ReflectError;

pub trait Error: for<'a> From<ReflectError<'a>> {
    type Operator;
    type Token;
    const NO_OPERATION: Self;
    const UNEXPECTED_COMMA: Self;

    fn missing_left_side(op: &Self::Operator) -> Self;
    fn missing_right_side(op: &Self::Operator) -> Self;
    fn unhandled_token(token: &Self::Token) -> Self;

    fn wrong_argument_count(
        function: String, 
        expected: usize, 
        received: usize,
    ) -> Self;
}
