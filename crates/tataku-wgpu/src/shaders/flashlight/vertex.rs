#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlashlightVertex {
    pub position: [f32; 2],
    pub flashlight_index: u32,
}
impl FlashlightVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // todo: convert to macro

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // flashlight index
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }
}