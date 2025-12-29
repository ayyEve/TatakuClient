use crate::*;
use tataku_common::reflect::*;

pub type Stack<'rpn, Output> = Vec<reflect::Result<'rpn, Output>>;

pub trait ShuntingYard<'rpn, 'values: 'rpn> {
    type Token: crate::Token<'values, Self::Output, Self::Error, Operator = Self::Operator>;
    type ReadType: Default;
    type Operator: crate::Operator<'values, Output = Self::Output, Error = Self::Error>;
    type Error: crate::Error<Operator = Self::Operator, Token = Self::Token>;
    type Output: 'values;

    fn read_check_char(
        read_type: &mut Self::ReadType,
        char: char,
        
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        function_arg_stack: &mut Vec<usize>,
    ) -> Result<bool, Self::Error>;

    fn add(
        read_type: &mut Self::ReadType,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        function_arg_stack: &mut Vec<usize>,
        is_open_paren: bool,
    ) -> Result<(), Self::Error>;
    
    fn resolve_token_value(
        token: &'rpn Self::Token,
        values: &'values dyn Reflect,
    ) -> Result<Self::Output, ReflectError<'rpn>>;

    fn run_function(
        function_token: &'rpn Self::Token, 
        stack: &mut Stack<'rpn, Self::Output>, 
        values: &'values dyn Reflect,
    ) -> Result<(), Self::Error>;


    fn open_paren(
        read_type: &mut Self::ReadType,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        function_arg_stack: &mut Vec<usize>,
    ) -> Result<(), Self::Error> {
        Self::add(
            read_type,
            output_queue, 
            operator_queue, 
            function_arg_stack,
            true,
        )?;

        if Self::last_is_fn(operator_queue) {
            function_arg_stack.push(1);
        }

        operator_queue.push(Self::Token::OPEN_PAREN);
        Ok(())
    }

    fn close_paren(
        read_type: &mut Self::ReadType,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        function_arg_stack: &mut Vec<usize>,
    ) -> Result<(), Self::Error> {
        Self::add(
            read_type,
            output_queue, 
            operator_queue, 
            function_arg_stack,
            false
        )?;

        while let Some(top) = operator_queue.pop() {
            if top == Self::Token::OPEN_PAREN { break }
            output_queue.push(top);
        }

        if Self::last_is_fn(operator_queue) {
            let arg_count = function_arg_stack.pop().unwrap();

            let mut last = operator_queue
                .pop()
                .unwrap();

            last.set_arg_count(arg_count);
            output_queue.push(last);
        }

        Ok(())
    }

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
                &mut function_arg_stack
            )?;

            if res { continue }
            
            match c {
                '(' => Self::open_paren(
                    &mut read_type, 
                    &mut output_queue, 
                    &mut operator_queue, 
                    &mut function_arg_stack
                )?,
                ')' => Self::close_paren(
                    &mut read_type, 
                    &mut output_queue, 
                    &mut operator_queue, 
                    &mut function_arg_stack
                )?,

                ',' => {
                    let Some(last) = function_arg_stack.last_mut()
                    else { return Err(Self::Error::UNEXPECTED_COMMA) };
                    *last += 1;
                    
                    Self::add(
                        &mut read_type,
                        &mut output_queue, 
                        &mut operator_queue, 
                        &mut function_arg_stack,
                        false
                    )?;
                }

                // check operators
                _ => match Self::Operator::read(c, c2) {
                    Err(_) if c == ' ' => {}
                    Err(OperatorReadError::Ignore) => {},

                    // TOdO:
                    Err(OperatorReadError::Unknown) 
                        => println!("Unknown operator char {c}"),

                    Ok(op) => {
                        Self::add(
                            &mut read_type,
                            &mut output_queue,
                            &mut operator_queue, 
                            &mut function_arg_stack,
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
            &mut function_arg_stack,
            false
        )?;

        while let Some(top) = operator_queue.pop() {
            output_queue.push(top);
        }


        Self::post_process_tokens(&mut output_queue);

        Ok(output_queue)
    }

    fn post_process_tokens(_tokens: &mut Vec<Self::Token>) {}
    fn post_process_resolved(_stack: &mut Stack<'rpn, Self::Output>) {}

    fn evaluate_rpn(
        rpn: &'rpn [Self::Token], 
        values: &'values dyn Reflect,
    ) -> Result<Self::Output, Self::Error> {
        let mut stack  = Stack::<'rpn, Self::Output>::new();

        for token in rpn {
            match token.get_type() {

                // if value, resolve and push to stack
                TokenType::Value => stack.push(
                    Self::resolve_token_value(
                        token,
                        values,
                    )
                ),
                
                // if is function, run function
                // if it has an output it should add it to the stack manually
                TokenType::Function 
                => Self::run_function(
                    token,
                    &mut stack,
                    values,
                )?,

                TokenType::Operator => {
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

        matches!(last.get_type(), TokenType::Function)
    }

    fn get_function_helper(
        function: &str,
        needed_arg_count: usize,
        provided_arg_count: usize,
        stack: &mut Stack<'rpn, Self::Output>,
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
