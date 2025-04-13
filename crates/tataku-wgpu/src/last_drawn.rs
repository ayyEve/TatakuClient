use tataku_engine::prelude::BlendMode;

// TODO: rename this
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LastDrawn {
    Standard,
    Slider,
    Flashlight,

    // special
    Blur,
}
impl LastDrawn {
    pub fn as_blendmode(self) -> BlendMode {
        match self {
            Self::Standard => BlendMode::AlphaBlending,
            Self::Flashlight => BlendMode::Flashlight,
            Self::Blur => BlendMode::Blur,
            Self::Slider => BlendMode::Slider,
        }
    }
}
