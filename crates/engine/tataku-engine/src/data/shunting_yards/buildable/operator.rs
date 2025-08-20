use crate::prelude::*;

#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BuildableShuntingYardOperator {
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

    // special
    Ref,
    Index,
}
impl<'values> _ShuntingYardOperator<'values> for BuildableShuntingYardOperator {
    type Output = Cow<'values, TatakuValue>;
    type Error = BuildableShuntingYardError;
    
    fn read(c1: char, c2: char) -> Result<Self, _ShuntingYardOperatorReadError> {
        match (c1, c2) {
            // math
            ('*', '*') => Ok(Self::Pow), 
            ('+', _) => Ok(Self::Add),
            ('-', _) => Ok(Self::Sub),
            ('*', _) => Ok(Self::Mul),
            ('/', _) => Ok(Self::Div),

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

            // special
            (':', ':') => Ok(Self::Ref),

            // ignorable errors
            ('|', _) | (_, '|') 
            | (':', _) | (_, ':') 
            | ('&', _) | (_, '&') 
            | ('=', _) | (_, '=') 
                => Err(_ShuntingYardOperatorReadError::Ignore),
            
            // err
            _ => Err(_ShuntingYardOperatorReadError::Unknown),
        }
    }

    fn perform(
        &self, 
        right: Self::Output, 
        left: Option<Self::Output>,
    ) -> Result<Self::Output, Self::Error> {
        // debug!("");
        // debug!("perform: {left:?} {self:?} {right:?}");
        let right = right.as_ref();

        let left = left
            .as_deref()
            .ok_or(Self::Error::MissingLeftSide(*self));

        let res = match self {
            // math
            Self::Add => left? + right,
            Self::Sub => left? - right,
            Self::Mul => left? * right,
            Self::Div => left? / right,
            Self::Pow => TatakuValue::F32(
                left?.as_f32().unwrap()
                    .powf(right.as_f32().unwrap())
            ),

            // math -> bool
            Self::Eq => TatakuValue::Bool(left? == right),
            Self::NotEq => TatakuValue::Bool(left? != right), 
            Self::Less => TatakuValue::Bool(left? < right), 
            Self::LessEq => TatakuValue::Bool(left? <= right), 
            Self::Greater => TatakuValue::Bool(left? > right), 
            Self::GreaterEq => TatakuValue::Bool(left? >= right), 

            // bool
            Self::And => TatakuValue::Bool(left?.as_bool() && right.as_bool()), //if left > 0.0 && right > 0.0 { 1.0 } else { 0.0 },
            Self::Or => TatakuValue::Bool(left?.as_bool() || right.as_bool()), //if left > 0.0 || right > 0.0 { 1.0 } else { 0.0 },
            Self::Not => TatakuValue::Bool(!right.as_bool()), //if right > 0.0 { 0.0 } else { 1.0 },
        
            // special
            Self::Ref => {
                let path = right.as_string();
                let path = ReflectPath::new(&path);
                match left? {
                    TatakuValue::Bool(b) => b.as_dyn().impl_get(path),
                    TatakuValue::F32(n) => n.as_dyn().impl_get(path),
                    TatakuValue::U32(n) => n.as_dyn().impl_get(path),
                    TatakuValue::U64(n) => n.as_dyn().impl_get(path),
                    TatakuValue::String(n) => n.as_dyn().impl_get(path),
                    TatakuValue::Reflect(n) => n.impl_get(path),
                    TatakuValue::None => Ok(MaybeOwnedReflect::Owned(Box::new(None::<u8>))),
                }
                .and_then(TatakuValue::from_reflection)
                .unwrap_or(TatakuValue::None)
            }

            Self::Index => left? + right,
        };
        // debug!("res: {res:?}");
        // debug!("");

        Ok(Cow::Owned(res))
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Ref | Self::Index => 6,
            Self::Pow => 5,
            Self::Mul | Self::Div => 4,
            Self::Add | Self::Sub => 3,

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
        !matches!(self, Self::Pow)
    }

    fn single_arg(&self) -> bool {
        matches!(self, Self::Not)
    }
}
