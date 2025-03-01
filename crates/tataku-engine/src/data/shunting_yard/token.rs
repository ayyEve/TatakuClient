#[derive(Debug, Clone)]
pub enum ShuntingYardToken {
    Number(f32),
    Operator(super::Operator),
    Variable(String),
    Function(String),
    LeftParenthesis,
    RightParenthesis,
}
