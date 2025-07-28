use crate::prelude::*;

type AstResult = Result<Vec<parsing::ShuntingYardToken>, ShuntingYardError>;

/// resolves variable paths, ie tmp.some_map::_something.id::game.blah
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(from="String", into="String")]
pub struct VariablePathResolver {
    pub(crate) var: String,
    ast: Arc<AstResult>,
}
impl VariablePathResolver {
    pub fn new(path: String) -> Self {
        let ast = Arc::new(
            parsing::ShuntingYard::parse_expression(&path)
        );
        
        Self {
            var: path,
            ast,
        }
    }

    pub fn resolve_path(&self, values: &dyn Reflect) -> TatakuResult<String> {
        match &*self.ast {
            Ok(rpn) => {
                let p = parsing::ShuntingYard::evaluate_rpn(
                    rpn, 
                    values
                )
                .map_err(|e| TatakuError::String(format!("{e:?}")))?;
                
                Ok(p)
            }
            Err(e) => Err(TatakuError::String(format!("{e:?}"))),
        }
    }
}
impl From<String> for VariablePathResolver {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
impl From<&str> for VariablePathResolver {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}
impl PartialEq for VariablePathResolver {
    fn eq(&self, other: &Self) -> bool {
        self.var.eq(&other.var)
    }
}
impl From<VariablePathResolver> for String {
    fn from(value: VariablePathResolver) -> Self {
        value.var
    }
}
 
mod parsing {
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq)]
    pub(super) enum ShuntingYardToken {
        Static(String),
        Reference,
        OpenParenthesis,
    }

    pub(super) struct ShuntingYard;
    impl ShuntingYard {
        pub fn parse_expression(
            expression: &str
        ) -> ShuntingYardResult<Vec<ShuntingYardToken>> {
            let mut output_queue: Vec<ShuntingYardToken> = Vec::new();
            let mut operator_queue: Vec<ShuntingYardToken> = Vec::new();

            let mut current_thing = CurrentThing::None;
            let expression = format!("{expression} ")
                .chars()
                .collect::<Vec<_>>();

            for pair in expression.windows(2) {
                let &[c, c2] = pair else { break };

                match c {
                    '0'..='9'
                    | 'a'..='z'
                    | 'A'..='Z'
                    | '.'
                    | '_' => current_thing.push(c),
                    

                    '(' => {
                        current_thing.add(&mut output_queue)?;
                        operator_queue.push(ShuntingYardToken::OpenParenthesis);
                    }

                    ')' => {
                        current_thing.add(&mut output_queue)?;

                        while let Some(top) = operator_queue.pop() {
                            if let ShuntingYardToken::OpenParenthesis = top { break }
                            output_queue.push(top);
                        }
                        
                        if let Some(ShuntingYardToken::Reference) = operator_queue.last() {
                            output_queue.push(operator_queue.pop().unwrap());
                        }
                    }


                    ':' => {
                        if c == c2 {
                            current_thing.add(&mut output_queue)?;
                            operator_queue.push(ShuntingYardToken::Reference);
                        }
                    }

                    _ => warn!("unknown char: {c}")
                }
            }

            // make sure to add the last thing if there is one
            current_thing.add(&mut output_queue)?;
            while let Some(top) = operator_queue.pop() {
                output_queue.push(top);
            }

            Ok(output_queue)
        }

        pub fn evaluate_rpn(
            rpn: &[ShuntingYardToken], 
            values: &dyn Reflect,
        ) -> ShuntingYardResult<String> {
            let mut stack = Vec::new();

            for token in rpn {
                match token {
                    ShuntingYardToken::Static(val) 
                        => stack.push(val.clone()),

                    ShuntingYardToken::Reference =>  {
                        let value = stack
                            .pop()
                            .ok_or(
                                ShuntingYardError::MissingLeftSide(Operator::Ref)
                            )?;

                        stack.push(values
                            .reflect_display(&value, None)?
                        );
                    }
                    _ => return Err(ShuntingYardError::InvalidType(
                        format!("{token:?}")
                    )),
                }
            }

            Ok(stack.join("."))
        }
    }

    /// Parsing helper
    enum CurrentThing {
        None,
        Static(String),
    }
    impl CurrentThing {
        fn push(&mut self, c: char) {
            match self {
                Self::Static(s) => s.push(c),
                Self::None => *self = Self::Static(c.to_string()),
            }
        }
        fn add(
            &mut self,
            output_queue: &mut Vec<ShuntingYardToken>,
        ) -> ShuntingYardResult<()>{
            match self {
                Self::None => return Ok(()),
                Self::Static(s) => output_queue.push(
                    ShuntingYardToken::Static(s.take().trim_matches('.').to_owned())
                ),
            }

            *self = Self::None;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_simple() {
            let str = "game.test.hi_123";

            let ast = ShuntingYard::parse_expression(str)
                .unwrap();
            // println!("{ast:#?}");

            assert_eq!(
                ast,
                vec![
                    ShuntingYardToken::Static("game.test.hi_123".into()),
                ]
            );

            let values = DynMap::default();
            let res = ShuntingYard::evaluate_rpn(
                &ast, 
                &values
            );

            assert_eq!(res, Ok(str.to_owned()));
        }

        #[test]
        fn test_reference() {
            let str = "game::game.test";
            let str_result = "game.hi";

            let ast = ShuntingYard::parse_expression(str)
                .unwrap();
            // println!("{ast:#?}");

            assert_eq!(
                ast,
                vec![
                    ShuntingYardToken::Static("game".into()),
                    ShuntingYardToken::Static("game.test".into()),
                    ShuntingYardToken::Reference,
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

            let res = ShuntingYard::evaluate_rpn(
                &ast, 
                &values
            );

            assert_eq!(res, Ok(str_result.to_owned()));
        }
    
    
        #[test]
        fn test_reference_paren() {
            let str = "game::(game.test).hi_123";
            let str_result = "game.hi.hi_123";

            let ast = ShuntingYard::parse_expression(str)
                .unwrap();
            // println!("{ast:#?}");

            assert_eq!(
                ast,
                vec![
                    ShuntingYardToken::Static("game".into()),
                    ShuntingYardToken::Static("game.test".into()),
                    ShuntingYardToken::Reference,
                    ShuntingYardToken::Static("hi_123".into()),
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

            let res = ShuntingYard::evaluate_rpn(
                &ast, 
                &values
            );

            assert_eq!(res, Ok(str_result.to_owned()));
        }
    
    }
    
}
