use tataku_engine::prelude::Pipeline;

// TODO: rename this
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LastPipeline {
    Standard,
    Slider,
    Flashlight,

    // special
    GaussianBlur,
    BoxBlur,
}
impl LastPipeline {
    pub fn as_blendmode(self) -> Pipeline {
        match self {
            Self::Standard => Pipeline::AlphaBlending,
            Self::Slider => Pipeline::Slider,
            Self::Flashlight => Pipeline::Flashlight,
            Self::GaussianBlur => Pipeline::GaussianBlur,
            Self::BoxBlur => Pipeline::BoxBlur,
        }
    }
}
