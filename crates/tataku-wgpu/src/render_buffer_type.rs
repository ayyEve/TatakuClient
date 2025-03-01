use crate::prelude::*;
use tataku_client_common::prelude::*;

pub enum RenderBufferType {
    Standard(Box<StandardBuffer>),
    Slider(Box<SliderRenderBuffer>),
    Flashlight(Box<FlashlightBuffer>),
}
impl RenderBufferType {
    pub fn get_scissor(&self) -> Scissor {
        match self {
            Self::Standard(v) => v.scissor.unwrap(),
            Self::Slider(s) => s.scissor.unwrap(),
            Self::Flashlight(f) => f.scissor.unwrap(),
        }
    }
    pub fn get_blend_mode(&self) -> BlendMode {
        match self {
            Self::Standard(v) => v.blend_mode,
            Self::Slider(_s) => BlendMode::Slider,
            Self::Flashlight(_f) => BlendMode::Flashlight,
        }
    }
    pub fn get_vertex_buffer(&self) -> &Buffer {
        match self {
            Self::Standard(v) => &v.vertex_buffer,
            Self::Slider(s) => &s.vertex_buffer,
            Self::Flashlight(f) => &f.vertex_buffer
        }
    }
    pub fn get_index_buffer(&self) -> &Buffer {
        match self {
            Self::Standard(v) => &v.index_buffer,
            Self::Slider(s) => &s.index_buffer,
            Self::Flashlight(f) => &f.index_buffer
        }
    }
    pub fn get_used_indices(&self) -> u64 {
        match self {
            Self::Standard(v) => v.used_indices,
            Self::Slider(s) => s.used_indices,
            Self::Flashlight(f) => f.used_indices,
        }
    }
}
