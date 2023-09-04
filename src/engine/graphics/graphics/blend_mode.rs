
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
    SourceAlphaBlending,

    /// special case
    Slider,
}
impl BlendMode {
    pub fn get_blend_state(&self) -> wgpu::BlendState {
        match self {
            Self::AlphaBlending => wgpu::BlendState::ALPHA_BLENDING,
            Self::AlphaOverwrite => wgpu::BlendState::REPLACE,
            Self::PremultipliedAlpha => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            Self::AdditiveBlending => wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
            },
            Self::SourceAlphaBlending => wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
            },

            _ => unimplemented!("nope")
        }
    }
}
