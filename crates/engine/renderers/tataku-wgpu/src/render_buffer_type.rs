use crate::shaders::*;
use crate::prelude::*;

pub enum RenderBufferType {
    Standard(Box<standard::Buffer>),
    Slider(Box<slider::Buffer>),
    Flashlight(Box<flashlight::Buffer>),
    GaussianBlur(Box<gaussian_blur::Buffer>),
    BoxBlur(Box<box_blur::Buffer>),

    #[cfg(feature="vello")]
    Vello(Box<vello::Buffer>),
}
impl RenderBufferType {
    pub fn get_scissor(&self) -> tataku::Scissor {
        match self {
            Self::Standard(v) => v.scissor.unwrap(),
            Self::Slider(s) => s.scissor.unwrap(),
            Self::Flashlight(f) => f.scissor.unwrap(),
            Self::GaussianBlur(b) => b.scissor.unwrap(),
            Self::BoxBlur(b) => b.scissor.unwrap(),

            #[cfg(feature="vello")]
            Self::Vello(b) => b.scissor.unwrap(),
        }
    }
    pub fn get_pipeline(&self) -> tataku::GraphicsPipeline {
        match self {
            Self::Standard(v) => v.blend_mode,
            Self::Slider(_) => tataku::GraphicsPipeline::Slider,
            Self::Flashlight(_) => tataku::GraphicsPipeline::Flashlight,
            Self::GaussianBlur(_) => tataku::GraphicsPipeline::GaussianBlur,
            Self::BoxBlur(_) => tataku::GraphicsPipeline::BoxBlur,

            #[cfg(feature="vello")]
            Self::Vello(_) => tataku::GraphicsPipeline::None,
        }
    }
    pub fn get_pipeline_type(&self) -> PipelineType {
        match self {
            Self::Standard(_) => PipelineType::Standard,
            Self::Slider(_) => PipelineType::Slider,
            Self::Flashlight(_) => PipelineType::Flashlight,
            Self::GaussianBlur(_) => PipelineType::GaussianBlur,
            Self::BoxBlur(_) => PipelineType::BoxBlur,

            #[cfg(feature="vello")]
            Self::Vello(_) => PipelineType::Vello,
        }
    }

    pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
        match self {
            Self::Standard(v) => &v.vertex_buffer,
            Self::Slider(s) => &s.vertex_buffer,
            Self::Flashlight(f) => &f.vertex_buffer,

            Self::GaussianBlur(_) => unimplemented!("no vertex buffer"),
            Self::BoxBlur(_) => unimplemented!("no vertex buffer"),

            #[cfg(feature="vello")]
            Self::Vello(_) => unimplemented!("no vertex buffer"),
        }
    }
    pub fn get_index_buffer(&self) -> &wgpu::Buffer {
        match self {
            Self::Standard(v) => &v.index_buffer,
            Self::Slider(s) => &s.index_buffer,
            Self::Flashlight(f) => &f.index_buffer,

            Self::GaussianBlur(_) => unimplemented!("no index buffer"),
            Self::BoxBlur(_) => unimplemented!("no index buffer"),

            #[cfg(feature="vello")]
            Self::Vello(_) => unimplemented!("no index buffer"),
        }
    }
    pub fn get_used_indices(&self) -> u64 {
        match self {
            Self::Standard(v) => v.used_indices,
            Self::Slider(s) => s.used_indices,
            Self::Flashlight(f) => f.used_indices,
            Self::GaussianBlur(_b) => unimplemented!("no used indices"),
            Self::BoxBlur(_b) => unimplemented!("no used indices"),

            #[cfg(feature="vello")]
            Self::Vello(f) => f.used,
        }
    }
}
