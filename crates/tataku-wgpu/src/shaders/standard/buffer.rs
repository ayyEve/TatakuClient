use crate::prelude::*;
use tataku_client_common::prelude::*;

const QUAD_PER_BUF:u64 = 3000;

pub struct StandardBuffer {
    pub blend_mode: BlendMode,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub scissor: Option<Scissor>,

    pub used_vertices: u64,
    pub used_indices: u64,
}
impl RenderBufferable for StandardBuffer {
    type Cache = CpuStandardBuffer;
    const VTX_PER_BUF: u64 = QUAD_PER_BUF * 4;
    const IDX_PER_BUF: u64 = QUAD_PER_BUF * 6;
    
    // fn name() -> &'static str { "vertex buffer" }
    fn should_write(&self) -> bool { self.used_indices > 0 }

    fn reset(&mut self) {
        self.blend_mode = BlendMode::None;
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;
    }

    fn dump(&mut self, queue: &Queue, cache: &Self::Cache) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&cache.cpu_vtx));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&cache.cpu_idx));
    }

    fn create_new_buffer(device: &Device) -> Self {
        StandardBuffer {
            blend_mode: BlendMode::None,
            scissor: None,
            vertex_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Vertex Buffer"),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                size: Self::VTX_PER_BUF * std::mem::size_of::<StandardVertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Index Buffer"),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                size: Self::IDX_PER_BUF * std::mem::size_of::<u32>() as u64,
                mapped_at_creation: false,
            }),
            used_vertices: 0,
            used_indices: 0,
        }
    }
}

pub struct CpuStandardBuffer {
    pub cpu_vtx: Vec<StandardVertex>,
    pub cpu_idx: Vec<u32>,
}
impl Default for CpuStandardBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![StandardVertex::default(); StandardBuffer::VTX_PER_BUF as usize],
            cpu_idx: vec![0; StandardBuffer::IDX_PER_BUF as usize],
        }
    }
}


pub struct StandardReserveData<'a> {
    pub vtx: &'a mut [StandardVertex],
    pub idx: &'a mut [u32],
    pub idx_offset: u64,
}
impl StandardReserveData<'_> {
    pub fn copy_in(&mut self, vtx: &[StandardVertex], idx: &[u32]) {
        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);
    }
}
