
// NOTE! if you add anything here be sure to implement the pipeline for it in the graphics engines!
#[allow(unused)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum Pipeline {
    /// dont use None to actually draw, its a helper used rendering side
    None,
    #[default]
    AlphaBlending,
    AlphaOverwrite,
    PremultipliedAlpha,
    AdditiveBlending,
    SourceAlphaBlending,

    /// because peppy stupid
    OsuAdditiveBlending,

    // special cases

    /// The slider shader
    Slider,

    /// The flashlight shader
    Flashlight,

    /// The gaussian blur shader
    GaussianBlur,

    /// The box blur shader
    BoxBlur,
}
impl Pipeline {
    pub fn is_blur(&self) -> bool {
        matches!(self, Self::GaussianBlur | Self::BoxBlur)
    }
}