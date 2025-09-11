use crate::*;
use common::reflect::*;
use tataku::{
    TatakuValue, 
    ShuntingYardStack, 
    GenericShuntingYard 
};
use engine::data::shunting_yards::{
    buildable::*,
    path_resolver::VariablePathResolver
};

pub struct BuildableShuntingYard;
impl<'rpn, 'values: 'rpn> BuildableShuntingYard {
    fn create_array(
        arg_count: usize,
        stack: &mut ShuntingYardStack<'rpn, <Self as GenericShuntingYard<'rpn,'values>>::Output>,
    ) -> Result<(), BuildableShuntingYardError> {
        let val = Self::get_function_helper(
            "create_array", 
            1, 
            arg_count, 
            stack
        )?.pop().unwrap();

        let val = val.as_string();

        let value = match &*val {
            "i32" => TatakuValue::Reflect(Box::new(Vec::<i32>::new())),
            "i64" => TatakuValue::Reflect(Box::new(Vec::<i64>::new())),
            "isize" => TatakuValue::Reflect(Box::new(Vec::<isize>::new())),
            
            "u32" => TatakuValue::Reflect(Box::new(Vec::<u32>::new())),
            "u64" => TatakuValue::Reflect(Box::new(Vec::<u64>::new())),
            "usize" => TatakuValue::Reflect(Box::new(Vec::<usize>::new())),

            "bool" => TatakuValue::Reflect(Box::new(Vec::<bool>::new())),
            "String" => TatakuValue::Reflect(Box::new(Vec::<String>::new())),

            other => return Err(BuildableShuntingYardError::ArgumentWrongType { 
                expected: "<primitive>".to_string(), 
                received: other.to_string(),
            }),
        };

        stack.push(Ok(Cow::Owned(value)));

        Ok(())
    }

    fn cast(
        arg_count: usize,
        stack: &mut ShuntingYardStack<'rpn, <Self as GenericShuntingYard<'rpn,'values>>::Output>,
    ) -> Result<(), BuildableShuntingYardError> {
        let mut args = Self::get_function_helper(
            "cast", 
            2, 
            arg_count, 
            stack
        )?;

        let value = args.pop().unwrap();
        let into_type = args.pop().unwrap().as_string();

        let Some(value) = value.as_f32() 
        else { 
            return Err(BuildableShuntingYardError::ArgumentWrongType { 
                expected: "<number>".to_string(), 
                received: value.type_name().to_string() 
            });
        };

        macro_rules! im_lazy {
            ($($t: ty,)*$(,)?) => {{
                match &*into_type {
                    $(
                        stringify!($t) => TatakuValue::Reflect(Box::new(value as $t)),
                    )*

                    other => {
                        return Err(BuildableShuntingYardError::ArgumentWrongType { 
                            expected: "<number type>".to_string(), 
                            received: other.to_string() 
                        });
                    }
                }
            }}
        }

        let value = im_lazy!(
            i8, i16, i32, i64, isize, i128, f32,
            u8, u16, u32, u64, usize, i128, f64,
        );

        stack.push(Ok(Cow::Owned(value)));
        Ok(())
    }

    fn math_function(
        arg_count: usize,
        function: MathFunction,
        stack: &mut ShuntingYardStack<'rpn, <Self as GenericShuntingYard<'rpn,'values>>::Output>,
    ) -> Result<(), BuildableShuntingYardError> {
        let val = Self::get_function_helper(
            function.str(), 
            1, 
            arg_count, 
            stack
        )?.pop().unwrap();

        stack.push(Ok(function.run(val)?));
        Ok(())
    }

    fn check_filter(
        arg_count: usize,
        values: &dyn Reflect,
        stack: &mut ShuntingYardStack<'rpn, <Self as GenericShuntingYard<'rpn,'values>>::Output>,
    ) -> Result<(), BuildableShuntingYardError> {
        let mut args = Self::get_function_helper(
            "check_filter", 
            2, 
            arg_count, 
            stack
        )?;

        let filter_var = args.pop().unwrap().as_string();
        let to_check = args.pop().unwrap().as_string();

        let filter = values.reflect_get::<engine::settings::ItemFilter>(
            &filter_var,
        )?;

        let result = filter.check(&to_check);
        stack.push(Ok(Cow::Owned(TatakuValue::Bool(result))));

        Ok(())
    }
}
impl<'rpn, 'values: 'rpn> GenericShuntingYard<'rpn, 'values> for BuildableShuntingYard {
    type Token = BuildableShuntingYardToken;
    type ReadType = BuildableShuntingYardReadType;
    type Operator = BuildableShuntingYardOperator;
    type Error = BuildableShuntingYardError;
    type Output = Cow<'values, TatakuValue>;

    fn read_check_char(
        read_type: &mut Self::ReadType,
        char: char,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        function_arg_stack: &mut Vec<usize>,
    ) -> Result<bool, Self::Error> {
        if let Self::ReadType::StringLiteral(s) = read_type {
            if char == '\'' {
                Self::add(
                    read_type,
                    output_queue, 
                    operator_queue,
                    function_arg_stack,
                    false
                )?;
            } else {
                s.push(char);
            }

            return Ok(true);
        }

        match char {
            '0'..='9'|'a'..='z'|'A'..='Z'|'.'|'_'|'['|']'|':' => {
                read_type.push(char);
                Ok(true)
            },

            '\'' => {
                *read_type = Self::ReadType::StringLiteral(String::new());
                Ok(true)
            },

            _ => Ok(false)
        }
    }

    fn add(
        read_type: &mut Self::ReadType,
        output_queue: &mut Vec<Self::Token>,
        operator_queue: &mut Vec<Self::Token>,
        _function_arg_stack: &mut Vec<usize>,
        is_open_paren: bool,
    ) -> Result<(), Self::Error>{
        match read_type {
            Self::ReadType::None => {},
            Self::ReadType::Number(s) => {
                let s = s.take();
                let number = s.parse::<f32>()
                    .map_err(|_| Self::Error::NumberIsntANumber(s))?;
                output_queue.push(Self::Token::Number(number));
            } 
            Self::ReadType::Variable(s) if is_open_paren => {
                // if current_thing is a variable, it is actually a function
                // this is because if there was an operation between it and this, current_thing should be none
                // ie "sin(123)" vs "sin + (123)"
                operator_queue.push(Self::Token::Function(s.take(), 1));
            }
            Self::ReadType::Variable(s) => {
                output_queue.push(Self::Token::Variable(s.take().into()));
            }
            Self::ReadType::StringLiteral(s) => {
                output_queue.push(Self::Token::StringLiteral(s.take()));
            }
        }

        *read_type = Self::ReadType::None;
        Ok(())
    }
    
    fn resolve_token_value(
        token: &'rpn Self::Token,
        values: &'values dyn Reflect,
    ) -> Result<Self::Output, ReflectError<'rpn>> {
        match token {
            BuildableShuntingYardToken::Number(num) 
                => Ok(Cow::Owned(TatakuValue::from(*num))),
            BuildableShuntingYardToken::StringLiteral(s) 
                => Ok(Cow::Owned(TatakuValue::from(s.clone()))),
            BuildableShuntingYardToken::Variable(var) => match &*var.var {
                "true"  => Ok(Cow::Owned(TatakuValue::Bool(true))),
                "false" => Ok(Cow::Owned(TatakuValue::Bool(false))),
                _var => {
                    let path = var.resolve_path(values).unwrap();

                    values
                        .impl_get(ReflectPath::new(&path))
                        .and_then(TatakuValue::from_reflection)
                        .map(Cow::Owned)
                        .map_err(ReflectError::to_owned)
                }
            },

            _ => unreachable!("token is not a value")
        }
    }

    fn run_function(
        function_token: &'rpn Self::Token, 
        stack: &mut ShuntingYardStack<'rpn, Self::Output>, 
        values: &'values dyn Reflect,
    ) -> Result<(), Self::Error> {
        let BuildableShuntingYardToken::Function(
            function, 
            arg_count
        ) = function_token else { unreachable!() };
        let arg_count = *arg_count;
        
        match &**function {
            "now" => stack.push(Ok(Cow::Owned(TatakuValue::F32(
                tataku::Instant::now().as_millis()))
            )),

            "abs" => Self::math_function(arg_count, MathFunction::Abs, stack)?,
            "sin" => Self::math_function(arg_count, MathFunction::Sin, stack)?,
            "cos" => Self::math_function(arg_count, MathFunction::Cos, stack)?,
            "tan" => Self::math_function(arg_count, MathFunction::Tan, stack)?,

            "floor" => Self::math_function(arg_count, MathFunction::Floor, stack)?,
            "round" => Self::math_function(arg_count, MathFunction::Round, stack)?,
            "ceil"  => Self::math_function(arg_count, MathFunction::Ceil, stack)?,

            "ref" => {
                let n = Self::get_function_helper(
                    "ref", 
                    1, 
                    arg_count, 
                    stack
                )?.pop().unwrap();

                let resolver = VariablePathResolver::new(n.as_string());
                let path = resolver
                    .resolve_path(values)
                    .map_err(|_| ReflectError::EntryNotExist { 
                        entry: format!("{resolver}").into() 
                    })?;

                let value = values
                    .impl_get(ReflectPath::new(&path))
                    .and_then(TatakuValue::from_reflection)
                    .unwrap_or(TatakuValue::None);
                // println!("ref({n:?}) = {value:?}");

                stack.push(Ok(Cow::Owned(value)));
            }

            "display" => {
                let n = Self::get_function_helper(
                    "display", 
                    1, 
                    1, 
                    stack
                )?.pop().unwrap();
                
                let mut precision = 2;
                if arg_count == 2 {
                    let n = stack
                        .pop()
                        .ok_or_else(|| 
                            BuildableShuntingYardError::MissingFunctionArgument(
                                function.to_owned()
                            )
                        )??;

                    precision = n
                        .as_u64()
                        .ok_or_else(|| 
                            BuildableShuntingYardError::ArgumentWrongType { 
                                expected: "number".to_owned(), 
                                received: n.type_name().to_owned(), 
                            }
                        )? 
                        as usize;
                }

                let str = match &*n {
                    TatakuValue::None => "None".to_owned(),
                    TatakuValue::F32(n) => tataku::format_float(n, precision),
                    TatakuValue::U32(n) => tataku::format_number(*n),
                    TatakuValue::U64(n) => tataku::format_number(*n),
                    TatakuValue::Bool(b) => format!("{b}"),
                    TatakuValue::String(s) => s.clone(),
                    TatakuValue::Reflect(reflect) 
                        => reflect
                            .reflect_display("", Some(precision))
                            .unwrap_or("?".to_owned()),
                };

                stack.push(Ok(Cow::Owned(TatakuValue::from(str))));
            }

            "type_name" => {
                let n = Self::get_function_helper(
                    "type_name", 
                    1, 
                    1, 
                    stack
                )?.pop().unwrap();

                let str = n.type_name();
                stack.push(Ok(Cow::Owned(TatakuValue::String(str.into()))));
            }

            "is_empty" => {
                let n = Self::get_function_helper(
                    "is_empty", 
                    1, 
                    arg_count, 
                    stack
                )?.pop().unwrap();
                stack.push(Ok(Cow::Owned(TatakuValue::from(n.is_empty()))));
            },

            "value_exists" => {
                let n = stack.pop();
                let exists = matches!(n, Some(Ok(_)));

                stack.push(Ok(Cow::Owned(TatakuValue::Bool(exists))));
            },

            "len" | "length" => {
                let n = Self::get_function_helper(
                    "length", 
                    1, 
                    1, 
                    stack
                )?.pop().unwrap();

                stack.push(Ok(Cow::Owned(TatakuValue::from(n.get_length() as u64))));
            },

            "array" => Self::create_array(arg_count, stack)?,
            "cast" => Self::cast(arg_count, stack)?,
            "check_filter" => Self::check_filter(arg_count, values, stack)?,

            other => return Err(BuildableShuntingYardError::InvalidFunction(other.to_string())),
        }
        
        Ok(())
    }
}


#[derive(Copy, Clone)]
enum MathFunction {
    Abs,
    Sin,
    Cos,
    Tan,

    Round,
    Ceil,
    Floor,
}
impl MathFunction {
    fn run(
        self, 
        val: Cow<'_, TatakuValue>
    ) -> Result<Cow<'_, TatakuValue>, BuildableShuntingYardError> {
        let num = val.as_number()
            .ok_or_else(|| BuildableShuntingYardError::NumberIsntANumber(val.as_string()))?;

        Ok(Cow::Owned(match self {
            Self::Abs => num.abs(),
            Self::Sin => num.sin(),
            Self::Cos => num.cos(),
            Self::Tan => num.tan(),

            Self::Round => num.round(),
            Self::Ceil => num.ceil(),
            Self::Floor => num.floor(),
        }.into()))
    }
    fn str(self) -> &'static str {
        match self {
            Self::Abs => "abs",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Round => "round",
            Self::Ceil => "ceil",
            Self::Floor => "floor",
        }
    }
}


#[cfg(test)]
mod shunting_yard_tests {
    use super::*;
    use tataku::TatakuValue;
    use tataku::GenericShuntingYard;

    #[test]
    fn reference_test() {
        let expression = "ref('::(b.path).a')";
        
        let values = DynMap::default()
            .set_chained("b.path", "c.path")
            .set_chained("c.path.a", 100u32)
        ;
    
        let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
        println!("Tokens: {tokens:?}");

        let result = BuildableShuntingYard::evaluate_rpn(&tokens, &values).unwrap();
        println!("Result: {result:?}");
        assert_eq!(*result, TatakuValue::U32(100));
        assert!(*result != TatakuValue::U32(10));
    }

    #[test]
    fn math_test() {
        let expression = "sin(test.0) + 4 * (2 - 7) / test.1 + 100.5";
        println!("expression: {expression}");

        let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
        println!("Tokens: {tokens:?}");

        let test:f32 = -30.0;
        let test_1:f32 = 50.0;

        let values = DynMap::default()
            .set_chained("test.0", test)
            .set_chained("test.1", test_1)
        ;
        let result = BuildableShuntingYard::evaluate_rpn(&tokens, &values).unwrap();
        println!("Result: {result:?}");
        let ok = TatakuValue::F32(test.sin() + 4.0 * (2.0 - 7.0) / test_1 + 100.5);
        assert_eq!(*result, ok);
    }


    #[test]
    fn bool_tests() {
        let expression = "100 == 100 && !(test.0 == test.1)";
        println!("Expression: {expression}");

        let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
        println!("Tokens: {tokens:?}");

        let test = -30.0;
        let test_1 = 50.0;

        let values = DynMap::default()
            .set_chained("test.0", test)
            .set_chained("test.1", test_1)
        ;

        let result = BuildableShuntingYard::evaluate_rpn(&tokens, &values).unwrap();
        println!("Result: {result:?}");
        assert_eq!(*result, TatakuValue::Bool(100 == 100 && !(test == test_1)));
    }

    #[test]
    fn single_bool_tests() {
        let expression = "test";
        println!("Expression: {expression}");

        let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
        println!("Tokens: {tokens:?}");

        let test = true;
        let values = DynMap::default()
            .set_chained("test", test)
        ;

        let result = BuildableShuntingYard::evaluate_rpn(&tokens, &values).unwrap();
        println!("Result: {result:?}");
        assert_eq!(*result, TatakuValue::Bool(test));
    }

    #[test]
    fn argument_count_tests() {

        // two arguments
        {
            let expression = "display(hi.mom, 3)";
            println!("Expression: {expression}");

            let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
            println!("Tokens: {tokens:?}");

            assert_eq!(
                tokens,
                vec![
                    BuildableShuntingYardToken::Variable("hi.mom".to_string().into()),
                    BuildableShuntingYardToken::Number(3.0),
                    BuildableShuntingYardToken::Function("display".to_string(), 2)
                ]
            );
        }

        // many arguments
        {
            let expression = "display(hi.mom, 1, 2, 3, 4, 5, 6)";
            println!("Expression: {expression}");

            let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
            println!("Tokens: {tokens:?}");

            assert_eq!(
                tokens,
                vec![
                    BuildableShuntingYardToken::Variable("hi.mom".to_string().into()),
                    BuildableShuntingYardToken::Number(1.0),
                    BuildableShuntingYardToken::Number(2.0),
                    BuildableShuntingYardToken::Number(3.0),
                    BuildableShuntingYardToken::Number(4.0),
                    BuildableShuntingYardToken::Number(5.0),
                    BuildableShuntingYardToken::Number(6.0),
                    BuildableShuntingYardToken::Function("display".to_string(), 7)
                ]
            );
        }
        
        // nested arguments
        {
            let expression = "display(hi.mom, calc(123, 'no u'))";
            println!("Expression: {expression}");
    
            let tokens = BuildableShuntingYard::parse_expression(expression).unwrap();
            println!("Tokens: {tokens:?}");
    
            assert_eq!(
                tokens,
                vec![
                    BuildableShuntingYardToken::Variable("hi.mom".to_string().into()),

                    // calc inner fn
                    BuildableShuntingYardToken::Number(123.0),
                    BuildableShuntingYardToken::StringLiteral("no u".to_string()),
                    BuildableShuntingYardToken::Function("calc".to_string(), 2),
                    
                    BuildableShuntingYardToken::Function("display".to_string(), 2),
                ]
            );
        }

    }


}
