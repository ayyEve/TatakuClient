use tataku_client_proc_macros::Default2;

// NOTE! if you add anything here be sure to implement the pipeline for it in the graphics engine(s)!
#[derive(Copy, Clone, Debug, Default2, Eq, PartialEq, Hash)]
pub enum GraphicsPipeline {
    None,
    
    #[default]
    Standard(BlendMode),

    /// The slider shader
    Slider,

    /// The flashlight shader
    Flashlight,

    /// The gaussian blur shader
    GaussianBlur,

    /// The box blur shader
    BoxBlur,
}
impl GraphicsPipeline {
    pub fn is_blur(&self) -> bool {
        matches!(self, Self::GaussianBlur | Self::BoxBlur)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum BlendMode {
    #[default]
    AlphaBlending,
    AlphaOverwrite,
    PremultipliedAlpha,
    AdditiveBlending,
    SourceAlphaBlending,

    /// because peppy stupid
    OsuAdditiveBlending,
}
impl BlendMode {
    pub const ALL: &[Self] = &[
        Self::AlphaBlending,
        Self::AlphaOverwrite,
        Self::PremultipliedAlpha,
        Self::AdditiveBlending,
        Self::SourceAlphaBlending,
        Self::OsuAdditiveBlending,
    ];
}

impl From<BlendMode> for GraphicsPipeline {
    fn from(value: BlendMode) -> Self {
        Self::Standard(value)
    }
}
