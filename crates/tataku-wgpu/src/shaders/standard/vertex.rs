use crate::prelude::*;
use tataku_client_common::prelude::Matrix;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct StandardVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub tex_index: i32,
    pub color: [f32; 4],
}
impl StandardVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<StandardVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                // tex coords
                VertexAttribute {
                    offset: std::mem::size_of::<[f32;2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                // tex index
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32;2]>() + std::mem::size_of::<[f32;2]>()) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Sint32,
                },
                // color
                VertexAttribute {
                    offset: (std::mem::size_of::<[f32;2]>() + std::mem::size_of::<[f32;2]>() + std::mem::size_of::<i32>()) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
            ]
        }
    }

    pub fn apply_matrix(mut self, matrix: &Matrix) -> Self {
        // matrix
        let pos = cgmath::Vector4::new(self.position[0], self.position[1], 0.0, 1.0);
        let new_pos = matrix * pos;
        self.position = [new_pos.x, new_pos.y];

        self
    }
}

impl Default for StandardVertex {
    fn default() -> Self {
        Self {
            position: [0.0; 2],
            tex_coords: [0.0; 2],
            tex_index: -1,
            color: [0.0; 4],
            // scissor_index: 0,
        }
    }
}
