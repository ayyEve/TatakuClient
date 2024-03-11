use crate::prelude::*;

pub struct ShuntingYard;
impl ShuntingYard {
    pub fn parse_expression(expression: &str) -> ShuntingYardResult<Vec<ShuntingYardToken>> {
        let mut output_queue: Vec<ShuntingYardToken> = Vec::new();
        let mut operator_stack: Vec<ShuntingYardToken> = Vec::new();

        let mut current_thing = CurrentThing::None;
        let expression = format!("{expression} ").chars().collect::<Vec<_>>();

        for pair in expression.windows(2) {
            let &[c, c2] = pair else { continue };

            match c {
                '0'..='9'|'a'..='z'|'.'|'_' => current_thing.push(c),

                // '+' | '-' | '*' | '/' | '^' => {
                //     current_thing.add(&mut output_queue, &mut operator_stack, false)?;
                //     while operator_stack.last().filter(|c2| Self::check_op(c, c2)).is_some() {
                //         output_queue.push(operator_stack.pop().unwrap());
                //     }
                //     operator_stack.push(ShuntingYardToken::Operator(c))
                // }

                '(' => {
                    // if current_thing is a variable, it is actually a function
                    // this is because if there was an operation between it and this, current_thing should be none
                    // ie "sin(123)" vs "sin + (123)"
                    current_thing.add(&mut output_queue, &mut operator_stack, true)?;
                    operator_stack.push(ShuntingYardToken::LeftParenthesis);
                }
                ')' => {
                    current_thing.add(&mut output_queue, &mut operator_stack, false)?;
                    while let Some(top) = operator_stack.pop() {
                        if let ShuntingYardToken::LeftParenthesis = top { break }
                        output_queue.push(top);
                    }

                    if let ShuntingYardToken::Function(_) = operator_stack.last().unwrap() {
                        output_queue.push(operator_stack.pop().unwrap());
                    }
                }
                
                _ => {
                    match SYOperator::from_chars(c, c2) {
                        // ignore warnings for space and equals
                        Err(ShuntingYardError::InvalidOperator(' '))
                        | Err(ShuntingYardError::InvalidOperator('=')) => {}

                        Err(e) => warn!("Error parsing operator {c}: {e:?}"),
                        Ok(op) => {
                            current_thing.add(&mut output_queue, &mut operator_stack, false)?;
        
                            while operator_stack.last().filter(|c2| Self::check_op(op, c2)).is_some() {
                                output_queue.push(operator_stack.pop().unwrap());
                            }

                            operator_stack.push(ShuntingYardToken::Operator(op));
                        } 
                    }
                }
            }
        }

        // make sure to add the last thing if there is one
        current_thing.add(&mut output_queue, &mut operator_stack, false)?;

        while let Some(top) = operator_stack.pop() {
            output_queue.push(top);
        }

        Ok(output_queue)
    }

    pub fn evaluate_rpn(rpn: &[ShuntingYardToken], values: &ShuntingYardValues) -> ShuntingYardResult<f32> {
        let mut stack = Vec::new();

        for token in rpn {
            match token {
                ShuntingYardToken::Number(num) => stack.push(*num),
                ShuntingYardToken::Variable(var) => stack.push(values.get_f32(&var)?),

                ShuntingYardToken::Function(func) => {
                    let n = stack.pop().ok_or(ShuntingYardError::MissingFunctionArgument(func.clone()))?;

                    match &**func {
                        "abs" => stack.push(n.abs()),
                        "sin" => stack.push(n.sin()),
                        "cos" => stack.push(n.cos()),
                        "tan" => stack.push(n.tan()),

                        other => return Err(ShuntingYardError::InvalidFunction(other.to_string())),
                    }
                }
                
                ShuntingYardToken::Operator(op) => {
                    let right = stack.pop().ok_or(ShuntingYardError::MissingRightSide(*op))?;
                    // "Not" is a special case, we only care about the right side
                    if let SYOperator::Not = op {
                        stack.push(op.perform(right, 0.0));
                        continue;
                    }

                    let left = stack.pop().ok_or(ShuntingYardError::MissingLeftSide(*op))?;
                    stack.push(op.perform(right, left));
                }

                _ => return Err(ShuntingYardError::InvalidToken(token.clone())),
            }
        }

        stack.pop().ok_or(ShuntingYardError::NoMath)
    }

    fn check_op(c1: SYOperator, c2: &ShuntingYardToken) -> bool {
        let ShuntingYardToken::Operator(c2) = c2 else { return false };
        let p1 = c1.precedence();
        let p2 = c2.precedence();
        (p2 > p1) || (p1 == p2 && c1.is_left_associative())
    }
}

/// helper for parsing numbers and variables
enum CurrentThing {
    None,
    Number(String),
    Variable(String),
}
impl CurrentThing {
    fn push(&mut self, c: char) {
        match self {
            Self::None => match c {
                '0'..='9' => *self = Self::Number(format!("{c}")),
                'a'..='z'|'.'|'_' => *self = Self::Variable(format!("{c}")),
                _ => {}
            }
            Self::Number(s) => s.push(c),
            Self::Variable(s) => s.push(c),
        }
    }
    fn add(
        &mut self, 
        output_queue: &mut Vec<ShuntingYardToken>, 
        operator_queue: &mut Vec<ShuntingYardToken>, 
        is_open_paren: bool,
    ) -> ShuntingYardResult<()>{
        match self {
            Self::None => return Ok(()),
            Self::Number(s) => {
                let s = s.take();
                let number = s.parse::<f32>()
                    .map_err(|_|ShuntingYardError::NumberIsntANumber(s))?;
                output_queue.push(ShuntingYardToken::Number(number));
            }
            Self::Variable(s) if is_open_paren => {
                operator_queue.push(ShuntingYardToken::Function(s.take()));
            }
            Self::Variable(s) => {
                output_queue.push(ShuntingYardToken::Variable(s.take()));
            }
        }

        *self = Self::None;
        Ok(())
    }
}


#[derive(Copy, Clone, Debug)]
pub enum SYOperator {
    // math
    Add,
    Sub,
    Mul,
    Div,
    Pow,

    // comparison
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    // bool
    And,
    Or,
    Not,
}
impl SYOperator {
    fn from_chars(c1: char, c2: char) -> ShuntingYardResult<Self> {
        match (c1, c2) {
            // math
            ('+', _) => Ok(Self::Add),
            ('-', _) => Ok(Self::Sub),
            ('*', _) => Ok(Self::Mul),
            ('/', _) => Ok(Self::Div),
            ('^', _) => Ok(Self::Pow),

            // comparison
            ('=', '=') => Ok(Self::Eq),
            ('!', '=') => Ok(Self::NotEq),
            ('<', '=') => Ok(Self::LessEq),
            ('<', _) => Ok(Self::Less),
            ('>', '=') => Ok(Self::GreaterEq),
            ('>', _) => Ok(Self::Greater),

            // bool
            ('&', '&') => Ok(Self::And),
            ('|', '|') => Ok(Self::Or),
            ('!', _) => Ok(Self::Not),

            // err
            _ => Err(ShuntingYardError::InvalidOperator(c1))
        }
    }

    fn perform(&self, right: f32, left: f32) -> f32 {
        match self {
            // math
            Self::Add => left + right,
            Self::Sub => left - right,
            Self::Mul => left * right,
            Self::Div => left / right,
            Self::Pow => left.powf(right),

            // math -> bool
            Self::Eq => if left == right { 1.0 } else { 0.0 },
            Self::NotEq => if left != right { 1.0 } else { 0.0 },
            Self::Less => if left < right { 1.0 } else { 0.0 },
            Self::LessEq => if left <= right { 1.0 } else { 0.0 },
            Self::Greater => if left > right { 1.0 } else { 0.0 },
            Self::GreaterEq => if left >= right { 1.0 } else { 0.0 },

            // bool
            Self::And => if left > 0.0 && right > 0.0 { 1.0 } else { 0.0 },
            Self::Or => if left > 0.0 || right > 0.0 { 1.0 } else { 0.0 },
            Self::Not => if right > 0.0 { 0.0 } else { 1.0 },
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Add | Self::Sub => 3,
            Self::Mul | Self::Div => 4,
            Self::Pow => 5,

            // comparisons should run after math
            Self::Eq | Self::NotEq 
            | Self::Less | Self::LessEq 
            | Self::Greater | Self::GreaterEq => 2,

            // bool logic should run after comparisons
            Self::Not => 1, // we want Not to run before And and Or
            Self::And | Self::Or => 0,
        }
    }

    fn is_left_associative(&self) -> bool {
        match self {
            Self::Pow => false,
            _ => true
        }
    }
}


#[allow(unused)]
mod shunting_yard_tests {
    use crate::prelude::*;

    #[test]
    fn math_test() {
        let expression = "sin(test) + 4 * (2 - 7) / test.1 + 100.5";
        println!("expression: {expression}");

        let tokens = ShuntingYard::parse_expression(expression).unwrap();
        println!("Tokens: {:?}", tokens);

        let test = -30.0;
        let test_1 = 50.0;

        let values = ShuntingYardValues::default()
            .set_chained("test", test)
            .set_chained("test.1", test_1)
        ;

        let result = ShuntingYard::evaluate_rpn(&tokens, &values).unwrap();
        println!("Result: {}", result);
        assert_eq!(result, test.sin() + 4.0 * (2.0 - 7.0) / test_1 + 100.5);
    }
    

    #[test]
    fn bool_tests() {
        let expression = "100 == 100 && !(test == test.1)";
        println!("Expression: {expression}");

        let tokens = ShuntingYard::parse_expression(expression).unwrap();
        println!("Tokens: {:?}", tokens);

        let test = -30.0;
        let test_1 = 50.0;

        let values = ShuntingYardValues::default()
            .set_chained("test", test)
            .set_chained("test.1", test_1)
        ;

        let result = ShuntingYard::evaluate_rpn(&tokens, &values).unwrap();
        println!("Result: {}", result);
        assert_eq!(result, 1.0);
    }
}
