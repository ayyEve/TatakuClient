use crate::prelude::*;


// TODO: nuke this now that we have reflect
#[derive(Debug, Default)]
pub enum TatakuValue {
    #[default] None,

    F32(f32),
    U32(u32),
    U64(u64),

    Bool(bool),
    String(String),
    Reflect(Box<dyn Reflect>),
}

impl<'de> serde::Deserialize<'de> for TatakuValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        use serde::de::Error;

        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = TatakuValue;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "one of: f32, f64, u16, u32, u64, bool, &str, String")
            }

            fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(v.into())
            }

            fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
                Ok(v.into())
            }
            fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok((v as f32).into())
            }

            fn visit_u16<E: Error>(self, v: u16) -> Result<Self::Value, E> {
                Ok((v as u32).into())
            }
            fn visit_u32<E: Error>(self, v: u32) -> Result<Self::Value, E> {
                Ok(v.into())
            }
            fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(v.into())
            }
            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                // FIXME: this is shit
                if let Ok(n) = v.parse::<u32>() {
                    Ok(n.into())
                } else if let Ok(n) = v.parse::<u64>() {
                    Ok(n.into())
                } else if let Ok(n) = v.parse::<f32>() {
                    Ok(n.into())
                } else if let Ok(n) = v.parse::<bool>() {
                    Ok(n.into())
                } 
                
                else {
                    Ok(v.into())
                }
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let key = map.next_key::<String>().unwrap();
                if key.is_none() {
                    return Ok(TatakuValue::None)
                }

                let val = map.next_value().unwrap_or_else(|e| panic!("error deserializng '{key:?}': {e:?}"));
                let _ = map.next_key::<String>().unwrap();
                
                Ok(val)
            }
                
        }

        deserializer.deserialize_any(Visitor)
    }
}


impl TatakuValue {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::None => false,
            Self::Bool(b) => *b,
            Self::U32(n) => *n != 0,
            Self::U64(n) => *n != 0,
            Self::F32(n) => *n > 0.0,
            Self::String(s) => !s.is_empty(),

            Self::Reflect(r) => if let Some(a) = r.downcast_ref::<bool>() {
                *a
            } else if let Ok(num) = r.reflect_as_number(ReflectPath::EMPTY) {
                let num:u64 = num.into();
                num != 0
            } else {
                false
            },
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Self::U32(i) => Some(*i as f32),
            Self::U64(i) => Some(*i as f32),
            Self::F32(f) => Some(*f),
            Self::Bool(b) => Some(*b as u8 as f32),

            Self::None => None,
            Self::String(s) => s.parse().ok(),
            
            Self::Reflect(r) => Some(
                r.reflect_as_number(ReflectPath::EMPTY)
                    .ok()?
                    .into()
            ),
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::U32(n) => Some(*n),
            Self::U64(n) => Some(*n as u32),
            Self::Reflect(r) => Some(
                r.reflect_as_number(ReflectPath::EMPTY)
                    .ok()?
                    .into()
            ),
            Self::String(s) => s.parse().ok(),

            Self::None => None,
            _ => None
        }
    }
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::None => None,
            Self::U32(n) => Some(*n as u64),
            Self::U64(n) => Some(*n),
            Self::Reflect(r) => Some(
                r.reflect_as_number(ReflectPath::EMPTY)
                    .ok()?
                    .into()
            ),
            Self::String(s) => s.parse().ok(),

            _ => None
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::None => "None".to_owned(),
            Self::U32(i) => i.to_string(),
            Self::U64(i) => i.to_string(),
            Self::F32(f) => format!("{f:.2}"),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => s.clone(),
            Self::Reflect(s) => s.reflect_display(
                ReflectPath::EMPTY, 
                None
            ).unwrap_or_else(|_| format!("No as_string! {}", s.type_name())),
        }
    }
    pub fn as_number(&self) -> Option<TatakuNumber> {
        match self {
            Self::F32(n) => Some(TatakuNumber::F32(*n)),
            Self::U32(n) => Some(TatakuNumber::U32(*n)),
            Self::U64(n) => Some(TatakuNumber::U64(*n)),
            Self::Reflect(r) => Some(r.reflect_as_number(".").ok()?.into()),
            
            Self::String(s) => Some(TatakuNumber::F32(s.parse::<f32>().ok()?)),
            _ => None
        }
    }

    pub fn from_reflection<'a>(value: impl Into<MaybeOwnedReflect<'a>>) -> Result<Self, ReflectError<'a>> {
        let value2: MaybeOwnedReflect<'a> = value.into();
        let value = value2.as_ref();

        if let Ok(n) = value.reflect_as_number(ReflectPath::EMPTY) {
            Ok(TatakuNumber::from(n).into())
        }
        else if let Some(b) = value.downcast_ref() {
            Ok(Self::Bool(*b))
        } else if let Some(s) = value.downcast_ref::<String>() {
            Ok(Self::String(s.clone()))
        } else if let Some(s) = value.downcast_ref::<Md5Hash>() {
            Ok(Self::String(s.to_string()))
        } 
        else if let Some(s) = value.downcast_ref::<GameSpeed>() {
            Ok(Self::F32(s.as_f32()))
        }
        else {
            match value2 {
                MaybeOwnedReflect::Owned(reflect) 
                    => Ok(Self::Reflect(reflect)),
                MaybeOwnedReflect::Borrowed(reflect) => reflect
                    .duplicate()
                    .map(Self::Reflect)
                    .ok_or(ReflectError::wrong_type(
                        value.type_name(), 
                        "TatakuValue"
                    )),
            }
        }
    }

    pub fn string_maybe(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            Self::Reflect(r) => r.downcast_ref::<String>(),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::String(s) => s.is_empty(),
            Self::Reflect(r) => r
                .reflect_display(".", None)
                .ok().as_ref()
                .map(String::is_empty)
                .unwrap_or_default(),
            
            _ => false,
        }
    }

    pub fn get_length(&self) -> usize {
        match self {
            Self::String(s) => s.len(),

            _ => 0
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Self::None => "None",
            Self::F32(_) => "f32",
            Self::U32(_) => "u32",
            Self::U64(_) => "u64",
            Self::Bool(_) => "Bool",
            Self::String(_) => "String",
            Self::Reflect(t) => t.type_name(),
        }
    }
}

impl Clone for TatakuValue {
    fn clone(&self) -> Self {
        match self {
            TatakuValue::None => Self::None,
            TatakuValue::F32(a) => Self::F32(*a),
            TatakuValue::U32(a) => Self::U32(*a),
            TatakuValue::U64(a) => Self::U64(*a),
            TatakuValue::Bool(a) => Self::Bool(*a),
            TatakuValue::String(a) => Self::String(a.clone()),
            TatakuValue::Reflect(a) => 
                a.duplicate().map(Self::Reflect).unwrap_or_default(),
        }
    }
}

impl PartialEq for TatakuValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::F32(n), Self::F32(n2)) => n == n2,
            (Self::U32(n), Self::U32(n2)) => n == n2,
            (Self::U64(n), Self::U64(n2)) => n == n2,
            (Self::Bool(n), Self::Bool(n2)) => n == n2,
            (Self::String(n), Self::String(n2)) => n == n2,

            (lhs, rhs) => {
                if let Some((lhs, rhs)) = lhs.as_number().zip(rhs.as_number()) {
                    lhs == rhs
                } else  {
                    lhs.as_string() == rhs.as_string()
                }
            }
        }
    }
}


impl strfmt::DisplayStr for TatakuValue {
    fn display_str(&self, f: &mut strfmt::Formatter) -> strfmt::Result<()> {
        match self {
            Self::U32(n) => n.display_str(f),
            Self::U64(n) => n.display_str(f),
            Self::F32(n) => n.display_str(f),
            Self::String(s) => s.display_str(f),
            Self::Bool(b) => f.str(if *b {"true"} else {"false"}),
            _ => f.str(&self.as_string()),
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

                    (TatakuValue::Reflect(lhs), rhs) => {
                        if let Some((lhs, rhs)) = lhs.reflect_as_number(".").ok().zip(rhs.as_number()) {
                            TatakuValue::from(TatakuNumber::from(lhs).$func(rhs))
                        } else {
                            TatakuValue::None
                        }
                    }
                    (lhs, TatakuValue::Reflect(rhs)) => {
                        if let Some((rhs, lhs)) = rhs.reflect_as_number(".").ok().zip(lhs.as_number()) {
                            TatakuValue::from(lhs.$func(TatakuNumber::from(rhs)))
                        } else {
                            TatakuValue::None
                        }
                    }

                    _ => panic!("nope")
                }
            }
        }
        
        impl std::ops::$trait for TatakuNumber {
            type Output = TatakuNumber;

            fn $func(self, rhs: Self) -> Self::Output {
                match (self, rhs) {
                    (Self::F32(lhs), Self::F32(rhs)) => Self::F32(lhs.$func(rhs)),
                    (Self::F32(lhs), Self::U32(rhs)) => Self::F32(lhs.$func(rhs as f32)),
                    (Self::F32(lhs), Self::U64(rhs)) => Self::F32(lhs.$func(rhs as f32)),

                    (Self::U32(lhs), Self::F32(rhs)) => Self::F32((lhs as f32).$func(rhs)),
                    (Self::U32(lhs), Self::U32(rhs)) => Self::U32(lhs.$func(rhs)),
                    (Self::U32(lhs), Self::U64(rhs)) => Self::U64((lhs as u64).$func(rhs)),
                    
                    (Self::U64(lhs), Self::F32(rhs)) => Self::F32((lhs as f32).$func(rhs)),
                    (Self::U64(lhs), Self::U32(rhs)) => Self::U64(lhs.$func(rhs as u64)),
                    (Self::U64(lhs), Self::U64(rhs)) => Self::U64(lhs.$func(rhs)),
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
impl PartialEq for TatakuNumber {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::F32(lhs), Self::F32(rhs)) => lhs == rhs,
            (Self::F32(lhs), Self::U32(rhs)) => lhs == rhs as f32,
            (Self::F32(lhs), Self::U64(rhs)) => lhs == rhs as f32,

            (Self::U32(lhs), Self::F32(rhs)) => lhs as f32 == rhs,
            (Self::U32(lhs), Self::U32(rhs)) => lhs == rhs,
            (Self::U32(lhs), Self::U64(rhs)) => lhs as u64 == rhs,
            
            (Self::U64(lhs), Self::F32(rhs)) => lhs as f32 == rhs,
            (Self::U64(lhs), Self::U32(rhs)) => lhs == rhs as u64,
            (Self::U64(lhs), Self::U64(rhs)) => lhs == rhs,
        }
    }
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

    pub fn round(&self) -> Self {
        if let Self::F32(n) = self {
            Self::U32(n.round() as u32)
        } else {
            *self
        }
    }
    pub fn floor(&self) -> Self {
        if let Self::F32(n) = self {
            Self::U32(n.floor() as u32)
        } else {
            *self
        }
    }
    pub fn ceil(&self) -> Self {
        if let Self::F32(n) = self {
            Self::U32(n.ceil() as u32)
        } else {
            *self
        }
    }

}

impl From<ReflectNumber> for TatakuNumber {
    fn from(value: ReflectNumber) -> Self {
        match value {
            ReflectNumber::U8(n) => Self::U32(n as u32),
            ReflectNumber::I8(n) => Self::U32(n as u32),
            ReflectNumber::U16(n) => Self::U32(n as u32),
            ReflectNumber::I16(n) => Self::U32(n as u32),
            ReflectNumber::U32(n) => Self::U32(n),
            ReflectNumber::I32(n) => Self::U32(n as u32),
            ReflectNumber::U64(n) => Self::U64(n),
            ReflectNumber::I64(n) => Self::U64(n as u64),
            ReflectNumber::U128(n) => Self::U64(n as u64),
            ReflectNumber::I128(n) => Self::U64(n as u64),
            ReflectNumber::Usize(n) => Self::U64(n as u64),
            ReflectNumber::Isize(n) => Self::U64(n as u64),
            ReflectNumber::F32(n) => Self::F32(n),
            ReflectNumber::F64(n) => Self::F32(n as f32),
            ReflectNumber::F16(n) => Self::F32(n.to_f32()),
            ReflectNumber::BF16(n) => Self::F32(n.to_f32()),
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
        impl<'a> TryFrom<&'a TatakuValue> for $t {
            type Error = TatakuValueError<'a>;

            fn try_from(value: &'a TatakuValue) -> Result<Self, Self::Error> {
                match value {
                    TatakuValue::$e(v) => Ok(v.clone()),
                    _ => Err(Self::Error::ValueWrongType {
                        expected: Cow::Borrowed(stringify!($t)),
                        received: Cow::Borrowed(value.type_name())
                    })
                }
            }
        }

        impl<'a> TryFrom<&'a TatakuValue> for &'a $t {
            type Error = TatakuValueError<'a>;

            fn try_from(value: &'a TatakuValue) -> Result<Self, Self::Error> {
                match value {
                    TatakuValue::$e(v) => Ok(v),
                    _ => Err(Self::Error::ValueWrongType {
                        expected: Cow::Borrowed(stringify!($t)),
                        received: Cow::Borrowed(value.type_name())
                    })
                }
            }
        }


    };

    ($t:ty, $e: ident, $t2: ty) => {
        impl From<$t> for TatakuValue {
            fn from(value: $t) -> Self { Self::$e(value as $t2) }
        }
        impl From<&$t> for TatakuValue {
            fn from(value: &$t) -> Self { Self::$e(value.clone() as $t2) }
        }

        impl<'a> TryFrom<&'a TatakuValue> for $t {
            type Error = TatakuValueError<'a>;

            fn try_from(value: &'a TatakuValue) -> Result<Self, Self::Error> {
                match value {
                    TatakuValue::$e(v) => Ok(*v as $t),
                    _ => Err(Self::Error::ValueWrongType {
                        expected: Cow::Borrowed(stringify!($t)),
                        received: Cow::Borrowed(value.type_name())
                    })
                }
            }
        }

    }
}
impl_from!(u8, U32, u32);
impl_from!(u16, U32, u32);
impl_from!(u32, U32);
impl_from!(u64, U64);
impl_from!(f32, F32);
impl_from!(f64, F32, f32);
impl_from!(bool, Bool);
impl_from!(String, String);

