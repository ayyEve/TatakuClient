use crate::prelude::*;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum TaikoHit {
    LeftKat,
    LeftDon,
    RightDon,
    RightKat
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitType {
    Don,
    Kat
}
impl From<KeyPress> for HitType {
    fn from(val: KeyPress) -> Self {
        match val {
            KeyPress::LeftKat|KeyPress::RightKat => HitType::Kat,
            KeyPress::LeftDon|KeyPress::RightDon => HitType::Don,
            _ => { panic!("non-taiko key while playing taiko") }
        }
    }
}