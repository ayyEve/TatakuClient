use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub tex_index: i32,
    pub color: [f32; 4],
    pub scissor_index: u32
}
impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
                    offset: std::mem::size_of::<[f32;2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex index
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32;2]>() + std::mem::size_of::<[f32;2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32;2]>() + std::mem::size_of::<[f32;2]>() + std::mem::size_of::<i32>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // scissor index
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32;2]>() + std::mem::size_of::<[f32;2]>() + std::mem::size_of::<i32>() + std::mem::size_of::<[f32;4]>()) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }

    pub fn apply_matrix(mut self, matrix: &Matrix) -> Self{
        // matrix
        let pos = cgmath::Vector4::new(self.position[0], self.position[1], 0.0, 1.0);
        let new_pos = matrix * pos;
        self.position = [new_pos.x, new_pos.y];

        self
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0; 2],
            tex_coords: [0.0; 2],
            tex_index: -1,
            color: [0.0; 4],
            scissor_index: 0,
        }
    }
}
