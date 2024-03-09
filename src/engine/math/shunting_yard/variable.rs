use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum CustomElementValue {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),

    F32(f32),
    String(String),
}
impl CustomElementValue {
    pub fn as_f32(&self) -> Result<f32, ShuntingYardError> {
        match self {
            CustomElementValue::I32(i) => Ok(*i as f32),
            CustomElementValue::I64(i) => Ok(*i as f32),
            CustomElementValue::U32(i) => Ok(*i as f32),
            CustomElementValue::U64(i) => Ok(*i as f32),
            CustomElementValue::F32(f) => Ok(*f),

            CustomElementValue::String(s) => Err(ShuntingYardError::ValueIsntANumber(s.clone()))
        } 
    }

    pub fn as_string(&self) -> String {
        match self {
            CustomElementValue::I32(i) => format!("{i}"),
            CustomElementValue::I64(i) => format!("{i}"),
            CustomElementValue::U32(i) => format!("{i}"),
            CustomElementValue::U64(i) => format!("{i}"),
            CustomElementValue::F32(f) => format!("{f:.2}"),
            CustomElementValue::String(s) => s.clone()
        } 
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
        }
    }
}


macro_rules! impl_from {
    ($t:ty, $e: ident) => {
        impl From<$t> for CustomElementValue {
            fn from(value: $t) -> Self { Self::$e(value) }
        }
    }
}

impl_from!(i32, I32);
impl_from!(i64, I64);
impl_from!(u32, U32);
impl_from!(u64, U64);
impl_from!(f32, F32);
impl_from!(String, String);
