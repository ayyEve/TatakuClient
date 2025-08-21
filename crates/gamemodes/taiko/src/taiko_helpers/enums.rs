use crate::prelude::*;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum TaikoHit {
    LeftKat,
    LeftDon,
    RightDon,
    RightKat
}
impl TaikoHit {
    pub const ALL: &[Self] = &[
        Self::LeftDon,
        Self::LeftKat,
        Self::RightDon,
        Self::RightKat,
    ];
}


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum HitType {
    #[default]
    Don,
    Kat
}
impl HitType {
    pub fn new(is_kat: bool) -> Self {
        if is_kat {
            Self::Kat
        } else {
            Self::Don
        }
    }
}
impl From<KeyPress> for HitType {
    fn from(val: KeyPress) -> Self {
        match val {
            KeyPress::LeftKat | KeyPress::RightKat => HitType::Kat,
            KeyPress::LeftDon | KeyPress::RightDon => HitType::Don,
            _ => { panic!("non-taiko key while playing taiko") }
        }
    }
}
impl std::ops::Not for HitType {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Don => Self::Kat,
            Self::Kat => Self::Don,
        }
    }
}