use crate::*;
use common::reflect::*;
use tataku::TatakuValue;

#[allow(unused)]
#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum GroupBy {
    #[default]
    Set,
    Collections,
}
impl GroupBy {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Set,
            Self::Collections,
        ]
    }
}
impl TryFrom<&TatakuValue> for GroupBy {
    type Error = String;
    fn try_from(value: &TatakuValue) -> Result<Self, Self::Error> {
        match value {
            TatakuValue::String(s) => {
                match &**s {
                    "Set" | "set" => Ok(Self::Set),
                    "Collections" | "collections" => Ok(Self::Collections),
                    other => Err(format!("invalid GroupBy str: '{other}'"))
                }
            }
            TatakuValue::U64(n) => {
                match *n {
                    0 => Ok(Self::Set),
                    1 => Ok(Self::Collections),
                    other => Err(format!("Invalid GroupBy number: {other}")),
                }
            }

            other => Err(format!("Invalid GroupBy value: {other:?}"))
        }
    }
}
impl From<GroupBy> for TatakuValue {
    fn from(val: GroupBy) -> Self {
        TatakuValue::String(format!("{val:?}"))
    }
}
