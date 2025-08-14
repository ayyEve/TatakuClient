use crate::prelude::*;
use tataku_client_common::prelude::*;

pub enum RenderBufferType {
    Standard(Box<StandardBuffer>),
    Slider(Box<SliderRenderBuffer>),
    Flashlight(Box<FlashlightBuffer>),
    GaussianBlur(Box<GaussianBlurBuffer>),
    BoxBlur(Box<BoxBlurBuffer>),
}
impl RenderBufferType {
    pub fn get_scissor(&self) -> Scissor {
        match self {
            Self::Standard(v) => v.scissor.unwrap(),
            Self::Slider(s) => s.scissor.unwrap(),
            Self::Flashlight(f) => f.scissor.unwrap(),
            Self::GaussianBlur(b) => b.scissor.unwrap(),
            Self::BoxBlur(b) => b.scissor.unwrap(),
        }
    }
    pub fn get_pipeline(&self) -> Pipeline {
        match self {
            Self::Standard(v) => v.blend_mode,
            Self::Slider(_) => Pipeline::Slider,
            Self::Flashlight(_) => Pipeline::Flashlight,
            Self::GaussianBlur(_) => Pipeline::GaussianBlur,
            Self::BoxBlur(_) => Pipeline::BoxBlur,
        }
    }
    pub fn get_vertex_buffer(&self) -> &Buffer {
        match self {
            Self::Standard(v) => &v.vertex_buffer,
            Self::Slider(s) => &s.vertex_buffer,
            Self::Flashlight(f) => &f.vertex_buffer,
            Self::GaussianBlur(_b) => unimplemented!("no vertex buffer"),
            Self::BoxBlur(_b) => unimplemented!("no vertex buffer"),
        }
    }
    pub fn get_index_buffer(&self) -> &Buffer {
        match self {
            Self::Standard(v) => &v.index_buffer,
            Self::Slider(s) => &s.index_buffer,
            Self::Flashlight(f) => &f.index_buffer,
            Self::GaussianBlur(_b) => unimplemented!("no index buffer"),
            Self::BoxBlur(_b) => unimplemented!("no index buffer"),
        }
    }
    pub fn get_used_indices(&self) -> u64 {
        match self {
            Self::Standard(v) => v.used_indices,
            Self::Slider(s) => s.used_indices,
            Self::Flashlight(f) => f.used_indices,
            Self::GaussianBlur(_b) => unimplemented!("no used indices"),
            Self::BoxBlur(_b) => unimplemented!("no used indices"),
        }
    }
}
