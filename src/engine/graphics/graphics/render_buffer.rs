use wgpu::Buffer;

pub struct RenderBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub used_vertices: u64,
    pub used_indices: u64,

    // recording_periods_since_last_use: usize
}
