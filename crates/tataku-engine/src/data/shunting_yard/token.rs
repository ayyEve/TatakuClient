use super::SYOperator;

#[derive(Debug, Clone)]
pub enum ShuntingYardToken {
    Number(f32),
    Operator(SYOperator),
    Variable(String),
    Function(String),
    LeftParenthesis,
    RightParenthesis,
}
