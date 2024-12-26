use crate::prelude::*;

#[derive(Debug, Default)]
pub enum TatakuValue {
    #[default]
    None,
    // I64(i64),
    F32(f32),
    U32(u32),
    U64(u64),

    Bool(bool),
    String(String),

    Reflect(Box<dyn Reflect>),

    List(Vec<TatakuVariable>),
    Map(HashMap<String, TatakuVariable>),
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
            Self::List(list) => !list.is_empty(),
            Self::String(s) => !s.is_empty(),
            Self::Map(m) => !m.is_empty(),

            Self::Reflect(r) => Self::from_reflection(&**r).map(|a| a.as_bool()).unwrap_or_default(),
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
            Self::Reflect(r) => Ok(Self::from_reflection(&**r)?.as_f32()?),
        }
    }

    pub fn as_u32(&self) -> Result<u32, ShuntingYardError> {
        match self {
            // Self::I32(n) => Ok(*n as u32),
            // Self::I64(n) => Ok(*n as u32),
            Self::U32(n) => Ok(*n),
            Self::U64(n) => Ok(*n as u32),
            Self::Reflect(r) => Ok(Self::from_reflection(&**r)?.as_u32()?),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            _ => Err(ShuntingYardError::ConversionError("Not castable to u32".to_string()))
        }
    }
    pub fn as_u64(&self) -> Result<u64, ShuntingYardError> {
        match self {
            // Self::I32(n) => Ok(*n as u64),
            // Self::I64(n) => Ok(*n as u64),
            Self::U32(n) => Ok(*n as u64),
            Self::U64(n) => Ok(*n),
            Self::Reflect(r) => Ok(Self::from_reflection(&**r)?.as_u64()?),

            Self::None => Err(ShuntingYardError::ValueIsNone),
            _ => Err(ShuntingYardError::ConversionError("Not castable to u64".to_string()))
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Self::None => "None".to_owned(),
            // Self::I32(i) => format!("{i}"),
            // Self::I64(i) => format!("{i}"),
            Self::U32(i) => format!("{i}"),
            Self::U64(i) => format!("{i}"),
            Self::F32(f) => format!("{f:.2}"),
            Self::Bool(b) => format!("{b}"),
            Self::String(s) => s.clone(),
            Self::Reflect(r) => Self::from_reflection(&**r).map(|a| a.as_string()).unwrap_or_default(),

            Self::List(a) => a.iter().map(|a| a.as_string()).collect::<Vec<_>>().join(" "),
            Self::Map(a) => a.iter().map(|(a, b)| format!("({a}: {})", b.as_string())).collect::<Vec<_>>().join(" "),
        }
    }
    pub fn as_number(&self) -> Option<TatakuNumber> {
        match self {
            Self::F32(n) => Some(TatakuNumber::F32(*n)),
            Self::U32(n) => Some(TatakuNumber::U32(*n)),
            Self::U64(n) => Some(TatakuNumber::U64(*n)),
            Self::Reflect(r) => Self::from_reflection(&**r).ok()?.as_number(),
            _ => None
        }
    }

    pub fn from_reflection<'a>(value: impl Into<MaybeOwnedReflect<'a>>) -> Result<Self, ReflectError<'a>> {
        let value = value.into();
        let value = value.as_ref();

        if let Some(n) = value.downcast_ref() {
            Ok(Self::F32(*n))
        } else if let Some(n) = value.downcast_ref() {
            Ok(Self::U32(*n))
        } else if let Some(n) = value.downcast_ref() {
            Ok(Self::U64(*n))
        } else if let Some(n) = value.downcast_ref::<usize>() {
            Ok(Self::U64(*n as u64))
        } else if let Some(n) = value.downcast_ref::<u8>() {
            Ok(Self::U32(*n as u32))
        } else if let Some(n) = value.downcast_ref::<u16>() {
            Ok(Self::U32(*n as u32))
        } else if let Some(b) = value.downcast_ref() {
            Ok(Self::Bool(*b))
        } else if let Some(s) = value.downcast_ref::<String>() {
            Ok(Self::String(s.clone()))
        } else if let Some(s) = value.downcast_ref::<Md5Hash>() {
            Ok(Self::String(s.to_string()))
        } 
        
        // TODO: figure out a way to do this automatically
        else if let Some(s) = value.downcast_ref::<ScoreRetreivalMethod>() {
            Ok(Self::String(s.to_string()))
        } else if let Some(s) = value.downcast_ref::<SortBy>() {
            Ok(Self::String(s.to_string()))
        }
        
        
        
        else {
            Err(ReflectError::wrong_type(value.type_name(), "TatakuValue"))
        }
    }


    pub fn to_map(self) -> HashMap<String, TatakuVariable> {
        let Self::Map(map) = self else { panic!("not a map"); };
        map
    }
    pub fn as_map(&self) -> Option<&HashMap<String, TatakuVariable>> {
        let Self::Map(map) = self else { return None };
        Some(map)
    }
    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, TatakuVariable>> {
        let Self::Map(map) = self else { return None };
        Some(map)
    }
    // pub fn as_map_helper(self) -> Option<ValueCollectionMapHelper> {
    //     let Self::Map(map) = self else { return None };
    //     Some(ValueCollectionMapHelper(map))
    // }

    pub fn string_maybe(&self) -> Option<&String> {
        match self {
            Self::String(s) => Some(s),
            Self::Reflect(r) => r.downcast_ref::<String>(),
            _ => None,
        }
    }
    pub fn list_maybe(&self) -> Option<&Vec<TatakuVariable>> {
        let Self::List(list) = self else { return None };
        Some(list)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::List(l) => l.is_empty(),
            Self::Map(m) => m.is_empty(),
            Self::String(s) => s.is_empty(),
            // Self::Reflect(r) => {r.}
            
            _ => false,
        }
    }

    pub fn get_length(&self) -> usize {
        match self {
            Self::List(l) => l.len(),
            Self::Map(m) => m.len(),
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
            Self::List(_) => "List",
            Self::Map(_) => "Map",
            Self::Reflect(_) => "Reflect",
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
            TatakuValue::List(l) => Self::List(l.clone()),
            TatakuValue::Map(m) => Self::Map(m.clone()),
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
            (Self::List(n), Self::List(n2)) => n == n2,
            (Self::Map(n), Self::Map(n2)) => n == n2,

            // TODO: ??
            (Self::Reflect(_), Self::Reflect(_)) => true,

            _ => false
        }
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
impl From<HashMap<String, TatakuVariable>> for TatakuValue {
    fn from(value: HashMap<String, TatakuVariable>) -> Self {
        Self::Map(value)
    }
}

impl From<TatakuValue> for Option<Box<dyn Reflect>> {
    fn from(val: TatakuValue) -> Self {
        match val {
            TatakuValue::Bool(b) => Some(Box::new(b)),
            TatakuValue::F32(n) => Some(Box::new(n)),
            TatakuValue::U32(n) => Some(Box::new(n)),
            TatakuValue::U64(n) => Some(Box::new(n)),
            TatakuValue::String(s) => Some(Box::new(s)),

            _ => None
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


impl<'lua> rlua::FromLua<'lua> for TatakuValue {
    fn from_lua(lua_value: rlua::Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        #[cfg(feature="debug_custom_menus")] info!("Reading TatakuValue");

        match &lua_value {
            rlua::Value::Boolean(b) => Ok(Self::Bool(*b)),
            // Value::Integer(i) => Ok(Self::I64(*i)),
            rlua::Value::Number(f) => Ok(Self::F32(*f as f32)),
            rlua::Value::String(s) => Ok(Self::String(s.to_str()?.to_owned())),
            // Value::Table(table) => {
            //     if let Ok(list) = table.get()
            // }
            other => Err(rlua::Error::FromLuaConversionError { from: other.type_name(), to: "TatakuValue", message: None }),
        }

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

        impl From<$t> for TatakuVariable {
            fn from(value: $t) -> Self { Self::new_game(TatakuValue::$e(value)) }
        }
        impl From<&$t> for TatakuVariable {
            fn from(value: &$t) -> Self { Self::new_game(TatakuValue::$e(value.clone())) }
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


        impl<'a> TryFrom<&'a TatakuVariable> for $t {
            type Error = TatakuValueError<'a>;

            fn try_from(value: &'a TatakuVariable) -> Result<Self, Self::Error> {
                match &value.value {
                    TatakuValue::$e(v) => Ok(v.clone()),
                    _ => Err(Self::Error::ValueWrongType {
                        expected: Cow::Borrowed(stringify!($t)),
                        received: Cow::Borrowed(value.type_name())
                    })
                }
            }
        }

        impl<'a> TryFrom<&'a TatakuVariable> for &'a $t {
            type Error = TatakuValueError<'a>;

            fn try_from(value: &'a TatakuVariable) -> Result<Self, Self::Error> {
                match &value.value {
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

        impl From<$t> for TatakuVariable {
            fn from(value: $t) -> Self { Self::new_game(TatakuValue::$e(value as $t2)) }
        }
        impl From<&$t> for TatakuVariable {
            fn from(value: &$t) -> Self { Self::new_game(TatakuValue::$e(value.clone() as $t2)) }
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


        impl<'a> TryFrom<&'a TatakuVariable> for $t {
            type Error = TatakuValueError<'a>;

            fn try_from(value: &'a TatakuVariable) -> Result<Self, Self::Error> {
                match &value.value {
                    TatakuValue::$e(v) => Ok(v.clone() as $t),
                    _ => Err(Self::Error::ValueWrongType {
                        expected: Cow::Borrowed(stringify!($t)),
                        received: Cow::Borrowed(value.type_name())
                    })
                }
            }
        }

    }
}
// impl_from!(i32, I32);
// impl_from!(i64, I64);
impl_from!(u8, U32, u32);
impl_from!(u16, U32, u32);
impl_from!(u32, U32);
impl_from!(u64, U64);
impl_from!(f32, F32);
impl_from!(f64, F32, f32);
impl_from!(bool, Bool);
impl_from!(String, String);

impl<T:Into<TatakuValue>> From<(TatakuVariableAccess, Vec<T>)> for TatakuValue {
    fn from((access, value): (TatakuVariableAccess, Vec<T>)) -> Self {
        Self::List(value.into_iter().map(|t| TatakuVariable::new(t.into()).access(access)).collect())
    }
}
impl<T:Into<TatakuValue>+Clone> From<(TatakuVariableAccess, &[T])> for TatakuValue {
    fn from((access, value): (TatakuVariableAccess, &[T])) -> Self {
        Self::List(value.iter().cloned().map(|t| TatakuVariable::new(t.into()).access(access)).collect())
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
        let TatakuValue::List(list) = value else { return Err("Value is not a list".to_string()) };

        let mut output = Vec::new();
        for i in list {
            output.push((&i.value).try_into().map_err(|e| e.to_string())?)
        }

        Ok(output)
    }
}

// #[derive(Default)]
// pub struct ValueCollectionMapHelper(HashMap<String, TatakuVariable>);
// impl ValueCollectionMapHelper {
//     pub fn set(&mut self, key: impl ToString, val: impl Into<TatakuVariable>) {
//         self.0.insert(key.to_string(), val.into());
//     }
//     pub fn insert(mut self, key: impl ToString, val: impl Into<TatakuVariable>) -> Self {
//         self.set(key, val);
//         self
//     }
//     pub fn finish(self) -> TatakuValue {
//         TatakuValue::Map(self.0)
//     }

//     pub fn try_get<'a, T: TryFrom<&'a TatakuValue, Error=TatakuValueError<'a>>>(&'a self, key: &str) -> Result<T, TatakuValueError> {
//         let entry = self.0.get(key).ok_or_else(|| TatakuValueError::EntryDoesntExist { entry: Cow::Owned(key.to_owned()) })?;
//         T::try_from(&entry.value)
//     }
// }


pub trait TatakuVariableMap {
    fn set_value(&mut self, key: impl ToString, val: impl Into<TatakuVariable>);
    fn insert_value(mut self, key: impl ToString, val: impl Into<TatakuVariable>) -> Self where Self:Sized {
        self.set_value(key, val);
        self
    }

    fn try_get<'a, T: TryFrom<&'a TatakuValue, Error=TatakuValueError<'a>>>(&'a self, key: &str) -> Result<T, TatakuValueError<'a>>;
}

impl TatakuVariableMap for HashMap<String, TatakuVariable> {
    fn set_value(&mut self, key: impl ToString, val: impl Into<TatakuVariable>) {
        self.insert(key.to_string(), val.into());
    }

    fn try_get<'a, T: TryFrom<&'a TatakuValue, Error=TatakuValueError<'a>>>(&'a self, key: &str) -> Result<T, TatakuValueError<'a>> {
        let entry = self.get(key).ok_or_else(|| TatakuValueError::EntryDoesntExist { entry: Cow::Owned(key.to_owned()) })?;
        T::try_from(&entry.value)
    }
}