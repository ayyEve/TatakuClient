use std::mem::size_of;
use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, tataku::Default2)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    #[default(-1)]
    pub tex_index: i32,
    pub color: [f32; 4],
}
impl Vertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: size_of::<[f32;2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex index
                wgpu::VertexAttribute {
                    offset: (
                        size_of::<[f32;2]>() 
                        + size_of::<[f32;2]>()
                    ) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (
                        size_of::<[f32;2]>() 
                        + size_of::<[f32;2]>() 
                        + size_of::<i32>()
                    ) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }

    pub fn apply_matrix(mut self, matrix: &tataku::Matrix) -> Self {
        // matrix
        let pos = cgmath::Vector4::new(
            self.position[0], 
            self.position[1], 
            0.0, 
            1.0
        );
        let new_pos = matrix * pos;
        self.position = [new_pos.x, new_pos.y];

        self
    }
}
