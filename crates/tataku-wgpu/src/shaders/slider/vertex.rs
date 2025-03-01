use crate::prelude::*;

/// Vertex buffer layout for sliders
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct SliderVertex {
    pub position: [f32; 2],
    pub slider_index: u32,
}
impl SliderVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                // slider index
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Uint32,
                },
            ]
        }
    }
}
