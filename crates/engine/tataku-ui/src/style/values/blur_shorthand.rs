use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct BlurShorthand {
    pub blur_amount: CssValue<f32>,
    pub blur_type: CssValue<CssBlurType>,
    pub blur_location: CssValue<BlurLocation>,
}
impl FromStr for BlurShorthand {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let split = s
            .split(" ")
            .collect::<Vec<_>>();

        match split.len() {
            0 => Ok(Self::default()),
            1 => Ok(Self {
                blur_amount: CssValue::parse_or_unset(split[0]),
                ..Default::default()
            }),
            2 => Ok(Self {
                blur_amount: CssValue::parse_or_unset(split[0]),
                blur_type: CssValue::parse_or_unset(split[1]),
                blur_location: CssValue::parse_or_unset(split[1]),
            }),
            3.. => Ok(Self {
                blur_amount: CssValue::parse_or_unset(split[0]),
                blur_type: CssValue::parse_or_unset(split[1]),
                blur_location: CssValue::parse_or_unset(split[2]),
            })
        }
    }
}
