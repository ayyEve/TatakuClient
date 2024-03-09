
#[derive(Debug, Clone)]
pub enum ShuntingYardToken {
    Number(f32),
    Operator(char),
    Variable(String),
    Function(String),
    LeftParenthesis,
    RightParenthesis,
}