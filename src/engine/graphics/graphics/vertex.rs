use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tex_index: i32,
    pub color: [f32; 4],
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tex coords
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex index
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32;3]>() + std::mem::size_of::<[f32;2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32;3]>() + std::mem::size_of::<[f32;2]>()+ std::mem::size_of::<i32>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }

    pub fn apply_matrix(mut self, matrix: &Matrix) -> Self{
        // matrix
        let pos = cgmath::Vector4::new(self.position[0], self.position[1], self.position[2], 1.0);
        let new_pos = matrix * pos;
        self.position = [new_pos.x, new_pos.y, new_pos.z];

        self
    }
}