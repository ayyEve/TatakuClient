use crate::prelude::tataku::GraphicsPipeline;

// TODO: rename this
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PipelineType {
    Standard,
    Slider,
    Flashlight,

    // special
    GaussianBlur,
    BoxBlur,

    #[cfg(feature="vello")]
    Vello,
}
impl PipelineType {
    pub fn is_blur(&self) -> bool {
        matches!(self, Self::GaussianBlur | Self::BoxBlur)
    }
    pub fn is_vello(&self) -> bool {
        matches!(self, Self::Vello)
    }

    pub fn special_render(&self) -> bool {
        self.is_blur() || self.is_vello()
    }

    pub fn as_pipeline(self) -> GraphicsPipeline {
        match self {
            Self::Standard => GraphicsPipeline::AlphaBlending,
            Self::Slider => GraphicsPipeline::Slider,
            Self::Flashlight => GraphicsPipeline::Flashlight,
            Self::GaussianBlur => GraphicsPipeline::GaussianBlur,
            Self::BoxBlur => GraphicsPipeline::BoxBlur,

            #[cfg(feature="vello")]
            Self::Vello => panic!("Trying to map Vello pipeline to GraphicsPipeline!")
        }
    }
}
