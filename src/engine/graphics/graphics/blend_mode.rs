
// NOTE! if you add anything here be sure to implement the pipeline for it in state.rs
#[allow(unused)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum BlendMode {
    /// dont use None to actually draw, its a helper used rendering side
    None,
    #[default]
    AlphaBlending,
    AlphaOverwrite,
    PremultipliedAlpha,
    AdditiveBlending,
}
