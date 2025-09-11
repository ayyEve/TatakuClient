use crate::*;
use common::reflect::*;

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum BlurLocation {
    Above,
    #[default] Below,
}
impl std::str::FromStr for BlurLocation {
    type Err = ();
    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "above" => Ok(Self::Above),
            "below" => Ok(Self::Below),
            _ => Err(()),
        }
    }
}
