use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum CustomElementValue {
    #[default]
    None,
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),

    F32(f32),

    Bool(bool),
    String(String),

    List(Vec<CustomElementValue>),
    Map(HashMap<String, CustomElementValue>),
}
impl CustomElementValue {
    pub fn is_none(&self) -> bool {
        if let Self::None = self {
            true
        } else {
            false
        }
    }

    pub fn as_f32(&self) -> Result<f32, ShuntingYardError> {
        match self {
            Self::I32(i) => Ok(*i as f32),
            Self::I64(i) => Ok(*i as f32),
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
            Self::I32(n) => Ok(*n as u32),
            Self::U32(n) => Ok(*n),
            Self::I64(n) => Ok(*n as u32),
            Self::U64(n) => Ok(*n as u32),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            _ => Err(ShuntingYardError::ConversionError(format!("Not castable to u32")))
        }
    }
    pub fn as_u64(&self) -> Result<u64, ShuntingYardError> {
        match self {
            Self::I32(n) => Ok(*n as u64),
            Self::U32(n) => Ok(*n as u64),
            Self::I64(n) => Ok(*n as u64),
            Self::U64(n) => Ok(*n),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            _ => Err(ShuntingYardError::ConversionError(format!("Not castable to u64")))
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::I32(i) => format!("{i}"),
            Self::I64(i) => format!("{i}"),
            Self::U32(i) => format!("{i}"),
            Self::U64(i) => format!("{i}"),
            Self::F32(f) => format!("{f:.2}"),
            Self::Bool(b) => format!("{b}"),
            Self::String(s) => s.clone(),

            Self::None => format!("None"),
            Self::List(a) => a.iter().map(|a| a.as_string()).collect::<Vec<_>>().join(" "),
            Self::Map(a) => a.iter().map(|(a, b)| format!("({a}: {})", b.as_string())).collect::<Vec<_>>().join(" "),
        } 
    }

    pub fn as_map(&self) -> Option<&HashMap<String, CustomElementValue>> {
        let Self::Map(map) = self else { return None };
        Some(map)
    }
    pub fn as_map_helper(self) -> Option<CustomElementMapHelper> {
        let Self::Map(map) = self else { return None };
        Some(CustomElementMapHelper(map))
    }


    pub fn string_maybe(&self) -> Option<&String> {
        let Self::String(s) = self else { return None };
        Some(s)
    }
    pub fn list_maybe(&self) -> Option<&Vec<Self>> {
        let Self::List(list) = self else { return None };
        Some(list)
    }
}
impl strfmt::DisplayStr for CustomElementValue {
    fn display_str(&self, f: &mut strfmt::Formatter) -> strfmt::Result<()> {
        match self {
            Self::I32(n) => n.display_str(f),
            Self::I64(n) => n.display_str(f),
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

impl From<&str> for CustomElementValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

macro_rules! impl_from {
    ($t:ty, $e: ident) => {
        impl From<$t> for CustomElementValue {
            fn from(value: $t) -> Self { Self::$e(value) }
        }
        impl From<&$t> for CustomElementValue {
            fn from(value: &$t) -> Self { Self::$e(value.clone()) }
        }
    }
}

impl_from!(i32, I32);
impl_from!(i64, I64);
impl_from!(u32, U32);
impl_from!(u64, U64);
impl_from!(f32, F32);
impl_from!(bool, Bool);
impl_from!(String, String);

impl<T:Into<CustomElementValue>> From<Vec<T>> for CustomElementValue {
    fn from(value: Vec<T>) -> Self {
        Self::List(value.into_iter().map(|t|t.into()).collect())
    }
}
impl<T:Into<CustomElementValue>+Clone> From<&[T]> for CustomElementValue {
    fn from(value: &[T]) -> Self {
        Self::List(value.into_iter().cloned().map(|t| t.into()).collect())
    }
}

impl<T:Into<CustomElementValue>> From<HashMap<String, T>> for CustomElementValue {
    fn from(value: HashMap<String, T>) -> Self {
        Self::Map(value.into_iter().map(|(k,v)| (k, v.into())).collect())
        // Self::List(value.into_iter().map(|t|t.into()).collect())
    }
}

impl<'a, T> TryFrom<&'a CustomElementValue> for Vec<T> 
where 
    &'a CustomElementValue: TryInto<T>, 
    <&'a CustomElementValue as TryInto<T>>::Error: ToString
{
    type Error = String;

    fn try_from(value: &'a CustomElementValue) -> Result<Self, Self::Error> {
        let CustomElementValue::List(list) = value else { return Err(format!("Value is not a list")) };

        let mut output = Vec::new();
        for i in list {
            output.push(i.try_into().map_err(|e| e.to_string())?)
        }

        Ok(output)
    }
}

#[derive(Default)]
pub struct CustomElementMapHelper(HashMap<String, CustomElementValue>);
impl CustomElementMapHelper {
    pub fn set(&mut self, key: impl ToString, val: impl Into<CustomElementValue>) {
        self.0.insert(key.to_string(), val.into());
    }
    pub fn finish(self) -> CustomElementValue {
        CustomElementValue::Map(self.0)
    }
}
