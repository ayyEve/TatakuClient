use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[derive(Reflect, Deserialize)]
#[serde(rename_all="camelCase")]
pub enum CssBlurType {
    #[default]
    Box,
    Gaussian,
}
impl CssBlurType {
    pub fn into_blur(self, amount: f32) -> BlurType {
        match self {
            Self::Box => BlurType::Box { size: amount.ceil() as u32 },
            Self::Gaussian => BlurType::Gaussian { sigma: amount.max(0.0) },
        }
    }
}
impl std::str::FromStr for CssBlurType {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "box" => Ok(Self::Box),
            "gaussian" => Ok(Self::Gaussian),
            _ => Err(()),
        }
    }
}

