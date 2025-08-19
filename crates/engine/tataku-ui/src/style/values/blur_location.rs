use crate::prelude::*;

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum BlurLocation {
    Above,
    #[default] Below,
}
impl std::str::FromStr for BlurLocation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "above" => Ok(Self::Above),
            "below" => Ok(Self::Below),
            _ => Err(()),
        }
    }
}
