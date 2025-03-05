#[derive(Debug, Clone)]
pub enum ShuntingYardToken {
    Number(f32),
    Operator(super::Operator),
    Variable(String),
    StringLiteral(String),
    Function(String),
    LeftParenthesis,
    RightParenthesis,
}
