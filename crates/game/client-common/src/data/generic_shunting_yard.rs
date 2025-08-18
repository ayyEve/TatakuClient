use tataku_common::prelude::*;

pub type ShuntingYardStack<'rpn, Output> = Vec<ReflectResult<'rpn, Output>>;

pub trait _ShuntingYardToken<'values, Output, Error>: PartialEq {
    type Operator: _ShuntingYardOperator<'values, Output = Output, Error = Error>;
    const OPEN_PAREN: Self;

    fn get_type(&self) -> _ShuntingYardTokenType;
    fn as_operator(&self) -> Option<&Self::Operator>;
    fn from_operator(op: Self::Operator) -> Self;
    fn set_arg_count(&mut self, count: usize);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum _ShuntingYardTokenType {
    Value,
    Function,
    Operator,
    Unknown,
}

pub trait _ShuntingYardOperator<'values>: Sized {
    type Output;
    type Error;

    fn read(c1: char, c2: char) -> Result<Self, _ShuntingYardOperatorReadError>;
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
pub enum _ShuntingYardOperatorReadError {
    Unknown,
    Ignore,
}

pub trait _ShuntingYardReadType: Default {}

pub trait _ShuntingYardError: for<'a> From<ReflectError<'a>> {
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

pub trait GenericShuntingYard<'rpn, 'values: 'rpn> {
    type Token: _ShuntingYardToken<'values, Self::Output, Self::Error, Operator = Self::Operator>;
    type ReadType: _ShuntingYardReadType;
    type Operator: _ShuntingYardOperator<'values, Output = Self::Output, Error = Self::Error>;
    type Error: _ShuntingYardError<Operator = Self::Operator, Token = Self::Token>;
    type Output: 'values;

    fn read_check_char(
        read_type: &mut Self::ReadType,
        char: char,
        
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
    ) -> Result<bool, Self::Error>;

    fn add(
        read_type: &mut Self::ReadType,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        is_open_paren: bool,
    ) -> Result<(), Self::Error>;
    
    fn resolve_token_value(
        token: &'rpn Self::Token,
        values: &'values dyn Reflect,
    ) -> Result<Self::Output, ReflectError<'rpn>>;

    fn run_function(
        function_token: &'rpn Self::Token, 
        stack: &mut ShuntingYardStack<'rpn, Self::Output>, 
        values: &'values dyn Reflect,
    ) -> Result<(), Self::Error>;

    fn parse_expression(
        expression: &str,
    ) -> Result<Vec<Self::Token>, Self::Error> {
        let mut output_queue: Vec<Self::Token> = Vec::new();
        let mut operator_queue: Vec<Self::Token> = Vec::new();

        let mut read_type = Self::ReadType::default();
        let expression = format!("{expression} ")
            .chars()
            .collect::<Vec<_>>();

        let mut function_arg_stack: Vec<usize> = Vec::new();


        for pair in expression.windows(2) {
            let &[c, c2] = pair else { continue };

            let res = Self::read_check_char(
                &mut read_type,
                c,
                &mut output_queue,
                &mut operator_queue,
            )?;

            if res { continue }
            
            match c {
                '(' => {
                    Self::add(
                        &mut read_type,
                        &mut output_queue, 
                        &mut operator_queue, 
                        true,
                    )?;

                    if Self::last_is_fn(&operator_queue) {
                        function_arg_stack.push(1);
                    }

                    operator_queue.push(Self::Token::OPEN_PAREN);
                }
                ')' => {
                    Self::add(
                        &mut read_type,
                        &mut output_queue, 
                        &mut operator_queue, 
                        false
                    )?;

                    while let Some(top) = operator_queue.pop() {
                        if top == Self::Token::OPEN_PAREN { break }
                        output_queue.push(top);
                    }

                    if Self::last_is_fn(&operator_queue) {
                        let arg_count = function_arg_stack.pop().unwrap();

                        let mut last = operator_queue
                            .pop()
                            .unwrap();

                        last.set_arg_count(arg_count);
                        output_queue.push(last);
                    }
                }
                ',' => {
                    let Some(last) = function_arg_stack.last_mut()
                    else { return Err(Self::Error::UNEXPECTED_COMMA) };
                    *last += 1;
                    
                    Self::add(
                        &mut read_type,
                        &mut output_queue, 
                        &mut operator_queue, 
                        false
                    )?;
                }

                // check operators
                _ => match Self::Operator::read(c, c2) {
                    Err(_) if c == ' ' => {}
                    Err(_ShuntingYardOperatorReadError::Ignore) => {},

                    // TOdO:
                    Err(_ShuntingYardOperatorReadError::Unknown) 
                        => println!("Unknown operator char {c}"),

                    Ok(op) => {
                        Self::add(
                            &mut read_type,
                            &mut output_queue,
                            &mut operator_queue, 
                            false
                        )?;

                        while operator_queue
                            .last()
                            .filter(|c2| Self::check_op(&op, c2))
                            .is_some()
                        {
                            output_queue.push(operator_queue.pop().unwrap());
                        }

                        operator_queue.push(Self::Token::from_operator(op));
                    }
                    
                }
            }
        }

        // make sure to add the last thing if there is one
        Self::add(
            &mut read_type,
            &mut output_queue, 
            &mut operator_queue, 
            false
        )?;

        while let Some(top) = operator_queue.pop() {
            output_queue.push(top);
        }


        Self::post_process_tokens(&mut output_queue);

        Ok(output_queue)
    }

    fn post_process_tokens(_tokens: &mut Vec<Self::Token>) {}
    fn post_process_resolved(_stack: &mut ShuntingYardStack<'rpn, Self::Output>) {}

    fn evaluate_rpn(
        rpn: &'rpn [Self::Token], 
        values: &'values dyn Reflect,
    ) -> Result<Self::Output, Self::Error> {
        let mut stack  = ShuntingYardStack::<'rpn, Self::Output>::new();

        for token in rpn {
            match token.get_type() {

                // if value, resolve and push to stack
                _ShuntingYardTokenType::Value => stack.push(
                    Self::resolve_token_value(
                        token,
                        values,
                    )
                ),
                
                // if is function, run function
                // if it has an output it should add it to the stack manually
                _ShuntingYardTokenType::Function 
                => Self::run_function(
                    token,
                    &mut stack,
                    values,
                )?,

                _ShuntingYardTokenType::Operator => {
                    let Some(
                        op
                    ) = token.as_operator()
                    else { panic!("token is operator but couldnt get op from token") };
                    
                    let right = stack
                        .pop()
                        .ok_or(Self::Error::missing_right_side(op))
                        ??;

                    if op.single_arg() {
                        stack.push(Ok(op.perform(
                            right, 
                            None
                        )?));
                    } else {
                        let left = stack
                            .pop()
                            .ok_or(Self::Error::missing_left_side(op))
                            ??;
                        stack.push(Ok(op.perform(right, Some(left))?));
                    }
                }

                _ => return Err(Self::Error::unhandled_token(token))
            }
        }


        Self::post_process_resolved(&mut stack);

        match stack.pop() {
            Some(Err(e)) => Err(e)?,
            Some(Ok(v)) => Ok(v),
            None => Err(Self::Error::NO_OPERATION)
        }
    }

    fn check_op(
        c1: &Self::Operator, 
        c2: &Self::Token,
    ) -> bool {
        let Some(c2) = c2.as_operator() 
        else { return false };

        let p1 = c1.precedence();
        let p2 = c2.precedence();
        (p2 > p1) || (p1 == p2 && c1.is_left_associative())
    }

    fn last_is_fn(op_queue: &[Self::Token]) -> bool {
        let Some(last) = op_queue.last() 
        else { return false };

        matches!(last.get_type(), _ShuntingYardTokenType::Function)
    }

    fn get_function_helper(
        function: &str,
        needed_arg_count: usize,
        provided_arg_count: usize,
        stack: &mut ShuntingYardStack<'rpn, Self::Output>,
    ) -> Result<Vec<Self::Output>, Self::Error> {

        if needed_arg_count != provided_arg_count {
            return Err(Self::Error::wrong_argument_count(
                function.to_owned(), 
                needed_arg_count, 
                provided_arg_count,
            ));
        }

        let list = 
            (0..needed_arg_count)
            .filter_map(|_| stack.pop())
            .collect::<Result<Vec<_>,_>>()?;

        Ok(list)
    }
}
