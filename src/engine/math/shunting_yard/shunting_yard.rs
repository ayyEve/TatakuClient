use crate::prelude::*;

pub struct ShuntingYard;
impl ShuntingYard {
    pub fn parse_expression(expression: &str) -> ShuntingYardResult<Vec<ShuntingYardToken>> {
        let mut output_queue: Vec<ShuntingYardToken> = Vec::new();
        let mut operator_stack: Vec<ShuntingYardToken> = Vec::new();

        let mut current_thing = CurrentThing::None;

        for c in expression.chars() {
            match c {
                '0'..='9'|'a'..='z'|'.'|'_' => current_thing.push(c),
                '+' | '-' | '*' | '/' | '^' => {
                    current_thing.add(&mut output_queue, &mut operator_stack, false)?;
 
                    while operator_stack.last().filter(|c2|Self::check_op(c, c2)).is_some() {
                        output_queue.push(operator_stack.pop().unwrap());
                    }
                    operator_stack.push(ShuntingYardToken::Operator(c))
                }
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

                _ => warn!("unknown character {c} in ")
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
                    let left = stack.pop().ok_or(ShuntingYardError::MissingLeftSide(*op))?;

                    match *op {
                        '+' => stack.push(left + right),
                        '-' => stack.push(left - right),
                        '*' => stack.push(left * right),
                        '/' => stack.push(left / right),
                        '^' => stack.push(left.powf(right)),
                        _ => return Err(ShuntingYardError::InvalidOperator(*op)),
                    }
                }

                _ => return Err(ShuntingYardError::InvalidToken(token.clone())),
            }
        }

        stack.pop().ok_or(ShuntingYardError::NoMath)
    }

    fn check_op(c1: char, c2:& ShuntingYardToken) -> bool {
        let ShuntingYardToken::Operator(c2) = c2 else { return false };
        if *c2 == '(' { return false }

        let Some(p1) = Self::precedence(c1) else { return false };
        let Some(p2) = Self::precedence(*c2) else { return false };
        (p2 > p1) || (p1 == p2 && Self::is_left_associative(c1))
    }
    fn precedence(c: char) -> Option<u8> {
        match c {
            '+' | '-' => Some(2),
            '*' | '/' => Some(3),
            '^' => Some(4),
            _ => None
        }
    }
    fn is_left_associative(c: char) -> bool {
        match c {
            '+' | '-' | '*' | '/' => true,
            '^' => false,
            _ => panic!("u wot m8")
        }
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



#[test]
fn test() {
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