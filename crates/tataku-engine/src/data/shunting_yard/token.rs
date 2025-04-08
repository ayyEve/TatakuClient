#[derive(Debug, Clone, PartialEq)]
pub enum ShuntingYardToken {
    Number(f32),
    Operator(super::Operator),
    Variable(String),
    StringLiteral(String),
    Function(String),
    LeftParenthesis,
    RightParenthesis,
}
