use crate::prelude::*;
use super::*;

pub(crate) struct PathShuntingYard;
impl<'rpn, 'values: 'rpn> GenericShuntingYard<'rpn, 'values> for PathShuntingYard {
    type Token = PathShuntingYardToken;
    type ReadType = PathShuntingYardReadType;
    type Operator = PathShuntingYardOperator;
    type Error = PathShuntingYardError;
    type Output = String;

    fn read_check_char(
        read_type: &mut Self::ReadType,
        char: char,
        _output_queue: &mut Vec<Self::Token>,
        _operator_queue: &mut Vec<Self::Token>,
    ) -> Result<bool, Self::Error> {
        match char {
            '0'..='9'|'a'..='z'|'A'..='Z'|'_'|'.' => {
                read_type.push(char);
                Ok(true)
            },

            _ => Ok(false),
        }
    }

    fn add(
        read_type: &mut Self::ReadType,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        _is_open_paren: bool,
    ) -> Result<(), Self::Error> {
        // any "operation" is really a function
        if matches!(operator_queue.last(), Some(Self::Token::Operation(PathShuntingYardOperator))) {
            operator_queue.pop();
            operator_queue.push(PathShuntingYardToken::Reference);
        }


        match read_type {
            Self::ReadType::None => return Ok(()),
            Self::ReadType::Static(s) => output_queue.push(
                Self::Token::Static(s.take().trim_matches('.').to_owned())
            ),
        }

        *read_type = Self::ReadType::None;
        Ok(())
    }

    fn resolve_token_value(
        token: &'rpn Self::Token,
        _values: &'values dyn Reflect,
    ) -> Result<Self::Output, ReflectError<'rpn>> {
        let PathShuntingYardToken::Static(s) = token 
        else { panic!("trying to resolve non-value token type") };
        Ok(s.clone())
    }

    fn run_function(
        function_token: &'rpn Self::Token, 
        stack: &mut ShuntingYardStack<'rpn, Self::Output>, 
        values: &'values dyn Reflect,
    ) -> Result<(), Self::Error> {
        let PathShuntingYardToken::Reference = function_token 
        else { panic!("trying to resolve non-reference token type") };

        let value = Self::get_function_helper(
            "reference", 
            1, 
            1, 
            stack
        )?.pop().unwrap();

        let value = values
            .reflect_display(&value, None)?;

        if let Some(Ok(top)) = stack.pop() {
            stack.push(Ok(format!("{top}.{value}")));
        } else {
            stack.push(Ok(value));
        }

        Ok(())
    }

    fn post_process_resolved(stack: &mut ShuntingYardStack<'rpn, Self::Output>) {
        let a = stack
            .take()
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
            .join(".");

        stack.push(Ok(a));
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let str = "game.test.hi_123";

        let ast = PathShuntingYard::parse_expression(str)
            .unwrap();
        // println!("{ast:#?}");

        assert_eq!(
            ast,
            vec![
                PathShuntingYardToken::Static("game.test.hi_123".into()),
            ]
        );

        let values = DynMap::default();
        let res = PathShuntingYard::evaluate_rpn(
            &ast, 
            &values
        );

        assert_eq!(res, Ok(str.to_owned()));
    }

    #[test]
    fn test_reference() {
        let str = "game::game.test";
        let str_result = "game.hi";

        let ast = PathShuntingYard::parse_expression(str)
            .unwrap();
        // println!("{ast:#?}");

        assert_eq!(
            ast,
            vec![
                PathShuntingYardToken::Static("game".into()),
                PathShuntingYardToken::Static("game.test".into()),
                PathShuntingYardToken::Reference,
            ]
        );

        let values = {
            let hi = DynMap::default()
                .set_chained("hi_123", "hello world")
                ;

            let game = DynMap::default()
                .set_chained("test", "hi")
                .set_chained("hi", hi)
                ;
            DynMap::default()
                .set_chained("game", game)
        };

        let res = PathShuntingYard::evaluate_rpn(
            &ast, 
            &values
        );

        assert_eq!(res, Ok(str_result.to_owned()));
    }

    #[test]
    fn test_reference2() {
        let str = "::game.test";
        let str_result = "hi";

        let ast = PathShuntingYard::parse_expression(str)
            .unwrap();
        // println!("{ast:#?}");

        assert_eq!(
            ast,
            vec![
                PathShuntingYardToken::Static("game.test".into()),
                PathShuntingYardToken::Reference,
            ]
        );

        let values = {
            let hi = DynMap::default()
                .set_chained("hi_123", "hello world")
                ;

            let game = DynMap::default()
                .set_chained("test", "hi")
                .set_chained("hi", hi)
                ;
            DynMap::default()
                .set_chained("game", game)
        };

        let res = PathShuntingYard::evaluate_rpn(
            &ast, 
            &values
        );

        assert_eq!(res, Ok(str_result.to_owned()));
    }


    #[test]
    fn test_reference_paren() {
        let str = "game::(game.test).hi_123";
        let str_result = "game.hi.hi_123";

        let ast = PathShuntingYard::parse_expression(str)
            .unwrap();
        println!("{ast:#?}");

        assert_eq!(
            ast,
            vec![
                PathShuntingYardToken::Static("game".into()),
                PathShuntingYardToken::Static("game.test".into()),
                PathShuntingYardToken::Reference,
                PathShuntingYardToken::Static("hi_123".into()),
            ]
        );

        let values = {
            let hi = DynMap::default()
                .set_chained("hi_123", "hello world")
                ;

            let game = DynMap::default()
                .set_chained("test", "hi")
                .set_chained("hi", hi)
                ;
            DynMap::default()
                .set_chained("game", game)
        };

        let res = PathShuntingYard::evaluate_rpn(
            &ast, 
            &values
        );

        assert_eq!(res, Ok(str_result.to_owned()));
    }

    #[test]
    fn test_reference_paren2() {
        let str = "::(game.test).hi_123";
        let str_result = "hi.hi_123";

        let ast = PathShuntingYard::parse_expression(str)
            .unwrap();
        println!("{ast:#?}");

        assert_eq!(
            ast,
            vec![
                PathShuntingYardToken::Static("game.test".into()),
                PathShuntingYardToken::Reference,
                PathShuntingYardToken::Static("hi_123".into()),
            ]
        );

        let values = {
            let hi = DynMap::default()
                .set_chained("hi_123", "hello world")
                ;

            let game = DynMap::default()
                .set_chained("test", "hi")
                .set_chained("hi", hi)
                ;
            DynMap::default()
                .set_chained("game", game)
        };

        let res = PathShuntingYard::evaluate_rpn(
            &ast, 
            &values
        );

        assert_eq!(res, Ok(str_result.to_owned()));
    }

}
