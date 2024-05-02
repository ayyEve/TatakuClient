use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TatakuValue {
    #[default]
    None,
    // I64(i64),
    F32(f32),
    U32(u32),
    U64(u64),

    Bool(bool),
    String(String),

    List(Vec<TatakuVariable>),
    Map(HashMap<String, TatakuVariable>),
}
impl TatakuValue {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::None => false,
            Self::Bool(b) => *b,
            Self::U32(n) => *n != 0,
            Self::U64(n) => *n != 0,
            Self::F32(n) => *n > 0.0,
            Self::List(list) => !list.is_empty(),
            Self::String(s) => !s.is_empty(),
            Self::Map(m) => !m.is_empty(),
        }
    }

    pub fn as_f32(&self) -> Result<f32, ShuntingYardError> {
        match self {
            // Self::I32(i) => Ok(*i as f32),
            // Self::I64(i) => Ok(*i as f32),
            Self::U32(i) => Ok(*i as f32),
            Self::U64(i) => Ok(*i as f32),
            Self::F32(f) => Ok(*f),
            Self::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            Self::String(s) => Err(ShuntingYardError::ValueIsntANumber(s.clone())),
            Self::List(_) => Err(ShuntingYardError::ValueIsntANumber("<vec>".to_owned())),
            Self::Map(_) => Err(ShuntingYardError::ValueIsntANumber("<map>".to_owned())),
        } 
    }

    pub fn as_u32(&self) -> Result<u32, ShuntingYardError> {
        match self {
            // Self::I32(n) => Ok(*n as u32),
            // Self::I64(n) => Ok(*n as u32),
            Self::U32(n) => Ok(*n),
            Self::U64(n) => Ok(*n as u32),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            _ => Err(ShuntingYardError::ConversionError(format!("Not castable to u32")))
        }
    }
    pub fn as_u64(&self) -> Result<u64, ShuntingYardError> {
        match self {
            // Self::I32(n) => Ok(*n as u64),
            // Self::I64(n) => Ok(*n as u64),
            Self::U32(n) => Ok(*n as u64),
            Self::U64(n) => Ok(*n),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            _ => Err(ShuntingYardError::ConversionError(format!("Not castable to u64")))
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::None => format!("None"),
            // Self::I32(i) => format!("{i}"),
            // Self::I64(i) => format!("{i}"),
            Self::U32(i) => format!("{i}"),
            Self::U64(i) => format!("{i}"),
            Self::F32(f) => format!("{f:.2}"),
            Self::Bool(b) => format!("{b}"),
            Self::String(s) => s.clone(),

            Self::List(a) => a.iter().map(|a| a.as_string()).collect::<Vec<_>>().join(" "),
            Self::Map(a) => a.iter().map(|(a, b)| format!("({a}: {})", b.as_string())).collect::<Vec<_>>().join(" "),
        } 
    }
    pub fn as_number(&self) -> Option<TatakuNumber> {
        match self {    
            Self::F32(n) => Some(TatakuNumber::F32(*n)),
            Self::U32(n) => Some(TatakuNumber::U32(*n)),
            Self::U64(n) => Some(TatakuNumber::U64(*n)),
            _ => None
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, TatakuVariable>> {
        let Self::Map(map) = self else { return None };
        Some(map)
    }
    pub fn as_map_helper(self) -> Option<ValueCollectionMapHelper> {
        let Self::Map(map) = self else { return None };
        Some(ValueCollectionMapHelper(map))
    }



    pub fn string_maybe(&self) -> Option<&String> {
        let Self::String(s) = self else { return None };
        Some(s)
    }
    pub fn list_maybe(&self) -> Option<&Vec<TatakuVariable>> {
        let Self::List(list) = self else { return None };
        Some(list)
    }
}
impl strfmt::DisplayStr for TatakuValue {
    fn display_str(&self, f: &mut strfmt::Formatter) -> strfmt::Result<()> {
        match self {
            // Self::I32(n) => n.display_str(f),
            // Self::I64(n) => n.display_str(f),
            Self::U32(n) => n.display_str(f),
            Self::U64(n) => n.display_str(f),
            Self::F32(n) => n.display_str(f),
            Self::String(s) => s.display_str(f),
            Self::Bool(b) => f.str(if *b {"true"} else {"false"}),
            _ => f.str(&self.as_string()),
            // Self::List(list) => f.str(&list.iter().map(|a|a.as_string()).collect::<Vec<_>>().join(" ")),
            // Self::Map(a) => f.str(&a.iter().map(|(a, b)| format!("({a}: {})", b.as_string())).collect::<Vec<_>>().join(" ")),
        }
    }
}

impl From<&str> for TatakuValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}
impl From<TatakuNumber> for TatakuValue {
    fn from(value: TatakuNumber) -> Self {
        match value {
            TatakuNumber::F32(n) => Self::F32(n),
            TatakuNumber::U32(n) => Self::U32(n),
            TatakuNumber::U64(n) => Self::U64(n),
        }
    }
}


macro_rules! impl_math {
    ($trait: ident, $func: ident) => {
        impl std::ops::$trait for &TatakuValue {
            type Output = TatakuValue;

            fn $func(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (TatakuValue::None, _) => TatakuValue::None,
                    (_, TatakuValue::None) => TatakuValue::None,

                    (TatakuValue::F32(lhs), TatakuValue::F32(rhs)) => TatakuValue::F32(lhs.$func(rhs)),
                    (TatakuValue::U32(lhs), TatakuValue::U32(rhs)) => TatakuValue::U32(lhs.$func(rhs)),
                    (TatakuValue::U64(lhs), TatakuValue::U64(rhs)) => TatakuValue::U64(lhs.$func(rhs)),

                    (TatakuValue::U32(lhs), TatakuValue::U64(rhs)) => TatakuValue::U64((*lhs as u64).$func(rhs)),
                    (TatakuValue::U64(lhs), TatakuValue::U32(rhs)) => TatakuValue::U64(lhs.$func(*rhs as u64)),
                    

                    (TatakuValue::U32(lhs), TatakuValue::F32(rhs)) => TatakuValue::F32((*lhs as f32).$func(rhs)),
                    (TatakuValue::U64(lhs), TatakuValue::F32(rhs)) => TatakuValue::F32((*lhs as f32).$func(rhs)),
                    
                    (TatakuValue::F32(lhs), TatakuValue::U32(rhs)) => TatakuValue::F32(lhs.$func(*rhs as f32)),
                    (TatakuValue::F32(lhs), TatakuValue::U64(rhs)) => TatakuValue::F32(lhs.$func(*rhs as f32)),
                    
                    // hopefully you arent doing other operations on a string
                    (TatakuValue::String(lhs), rhs) => TatakuValue::String(format!("{lhs}{}", &rhs.as_string())),
                    (lhs, TatakuValue::String(rhs)) => TatakuValue::String(format!("{}{rhs}", lhs.as_string())),


                    _ => panic!("nope") 
                }
            }
        }
    };
}

impl_math!(Add, add);
impl_math!(Sub, sub);
impl_math!(Mul, mul);
impl_math!(Div, div);
impl_math!(Rem, rem);

impl PartialOrd for TatakuValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;

        Some(match (self, other) {
            (TatakuValue::None, _) => Ordering::Equal,
            (_, TatakuValue::None) => Ordering::Equal,

            (TatakuValue::F32(lhs), TatakuValue::F32(rhs)) => lhs.partial_cmp(rhs).unwrap_or(Ordering::Equal),
            (TatakuValue::U32(lhs), TatakuValue::U32(rhs)) => lhs.cmp(rhs),
            (TatakuValue::U64(lhs), TatakuValue::U64(rhs)) => lhs.cmp(rhs),

            (TatakuValue::U32(lhs), TatakuValue::U64(rhs)) => (*lhs as u64).cmp(rhs),
            (TatakuValue::U64(lhs), TatakuValue::U32(rhs)) => lhs.cmp(&(*rhs as u64)),
            

            (TatakuValue::U32(lhs), TatakuValue::F32(rhs)) => (*lhs as f32).partial_cmp(rhs).unwrap_or(Ordering::Equal),
            (TatakuValue::U64(lhs), TatakuValue::F32(rhs)) => (*lhs as f32).partial_cmp(rhs).unwrap_or(Ordering::Equal),
            
            (TatakuValue::F32(lhs), TatakuValue::U32(rhs)) => lhs.partial_cmp(&(*rhs as f32)).unwrap_or(Ordering::Equal),
            (TatakuValue::F32(lhs), TatakuValue::U64(rhs)) => lhs.partial_cmp(&(*rhs as f32)).unwrap_or(Ordering::Equal),
            
            // // hopefully you arent doing other operations on a string
            // (TatakuValue::String(lhs), rhs) => TatakuValue::String(format!("{lhs}{}", &rhs.as_string())),
            // (lhs, TatakuValue::String(rhs)) => TatakuValue::String(format!("{}{rhs}", lhs.as_string())),


            _ => Ordering::Equal
        })
    }
}


#[derive(Copy, Clone, Debug)]
pub enum TatakuNumber {
    F32(f32),
    U32(u32),
    U64(u64),
}
impl TatakuNumber {
    pub fn cos(&self) -> Self {
        match self {
            Self::F32(n) => Self::F32(n.cos()),
            Self::U32(n) => Self::F32((*n as f32).cos()),
            Self::U64(n) => Self::F32((*n as f32).cos()),
        }
    }
    pub fn sin(&self) -> Self {
        match self {
            Self::F32(n) => Self::F32(n.sin()),
            Self::U32(n) => Self::F32((*n as f32).sin()),
            Self::U64(n) => Self::F32((*n as f32).sin()),
        }
    }    
    pub fn tan(&self) -> Self {
        match self {
            Self::F32(n) => Self::F32(n.sin()),
            Self::U32(n) => Self::F32((*n as f32).sin()),
            Self::U64(n) => Self::F32((*n as f32).sin()),
        }
    }
    
    pub fn abs(&self) -> Self {
        match self {
            Self::F32(n) => Self::F32(n.abs()),
            Self::U32(n) => Self::U32(*n),
            Self::U64(n) => Self::U64(*n),
        }
    }

}





macro_rules! impl_from {
    ($t:ty, $e: ident) => {
        impl From<$t> for TatakuValue {
            fn from(value: $t) -> Self { Self::$e(value) }
        }
        impl From<&$t> for TatakuValue {
            fn from(value: &$t) -> Self { Self::$e(value.clone()) }
        }
    }
}
// impl_from!(i32, I32);
// impl_from!(i64, I64);
impl_from!(u32, U32);
impl_from!(u64, U64);
impl_from!(f32, F32);
impl_from!(bool, Bool);
impl_from!(String, String);

impl<T:Into<TatakuValue>> From<(TatakuVariableAccess, Vec<T>)> for TatakuValue {
    fn from((access, value): (TatakuVariableAccess, Vec<T>)) -> Self {
        Self::List(value.into_iter().map(|t| TatakuVariable::new(t.into()).access(access)).collect())
    }
}
impl<T:Into<TatakuValue>+Clone> From<(TatakuVariableAccess, &[T])> for TatakuValue {
    fn from((access, value): (TatakuVariableAccess, &[T])) -> Self {
        Self::List(value.into_iter().cloned().map(|t| TatakuVariable::new(t.into()).access(access)).collect())
        // Self::List(value.into_iter().cloned().map(|t| t.into()).collect())
    }
}

// impl<T:Into<TatakuValue>> From<HashMap<String, T>> for TatakuValue {
//     fn from(value: HashMap<String, T>) -> Self {
//         Self::Map(value.into_iter().map(|(k,v)| (k, v.into())).collect())
//         // Self::List(value.into_iter().map(|t|t.into()).collect())
//     }
// }

impl<'a, T> TryFrom<&'a TatakuValue> for Vec<T> 
where 
    &'a TatakuValue: TryInto<T>, 
    <&'a TatakuValue as TryInto<T>>::Error: ToString
{
    type Error = String;

    fn try_from(value: &'a TatakuValue) -> Result<Self, Self::Error> {
        let TatakuValue::List(list) = value else { return Err(format!("Value is not a list")) };

        let mut output = Vec::new();
        for i in list {
            output.push((&i.value).try_into().map_err(|e| e.to_string())?)
        }

        Ok(output)
    }
}

#[derive(Default)]
pub struct ValueCollectionMapHelper(HashMap<String, TatakuVariable>);
impl ValueCollectionMapHelper {
    pub fn set(&mut self, key: impl ToString, val: impl Into<TatakuVariable>) {
        self.0.insert(key.to_string(), val.into());
    }
    pub fn finish(self) -> TatakuValue {
        TatakuValue::Map(self.0)
    }
}




