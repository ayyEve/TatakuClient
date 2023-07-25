use super::BlendMode;
use wgpu::{Buffer, BindGroup};

pub struct RenderBuffer {
    pub blend_mode: BlendMode,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub scissor_buffer: Buffer,
    pub scissor_buffer_bind_group: BindGroup,

    pub used_vertices: u64,
    pub used_indices: u64,
    pub used_scissors: u64,

    // recording_periods_since_last_use: usize
}
