
use crate::prelude::*;

use tataku_client_common::prelude::*;
use wgpu::{
    Buffer,
    Device
};

const QUAD_PER_BUF:u64 = 3000;
const VTX_PER_BUF:u64 = QUAD_PER_BUF * 4;
const IDX_PER_BUF:u64 = QUAD_PER_BUF * 6;

pub struct VertexBuffer {
    pub blend_mode: BlendMode,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub scissor: Option<Scissor>,

    pub used_vertices: u64,
    pub used_indices: u64,
}
impl RenderBufferable for VertexBuffer {
    type Cache = CpuVertexBuffer;
    const VTX_PER_BUF: u64 = VTX_PER_BUF;
    const IDX_PER_BUF: u64 = IDX_PER_BUF;
    
    // fn name() -> &'static str { "vertex buffer" }
    fn should_write(&self) -> bool { self.used_indices > 0 }

    fn reset(&mut self) {
        self.blend_mode = BlendMode::None;
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&cache.cpu_vtx));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&cache.cpu_idx));
    }

    fn create_new_buffer(device: &Device) -> Self {
        VertexBuffer {
            blend_mode: BlendMode::None,
            scissor: None,
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: VTX_PER_BUF * std::mem::size_of::<Vertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size: IDX_PER_BUF * std::mem::size_of::<u32>() as u64,
                mapped_at_creation: false,
            }),
            used_vertices: 0,
            used_indices: 0,
        }
    }
}

pub struct CpuVertexBuffer {
    pub cpu_vtx: Vec<Vertex>,
    pub cpu_idx: Vec<u32>,
}
impl Default for CpuVertexBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![Vertex::default(); VTX_PER_BUF as usize],
            cpu_idx: vec![0; IDX_PER_BUF as usize],
        }
    }
}


pub struct VertexReserveData<'a> {
    pub vtx: &'a mut [Vertex],
    pub idx: &'a mut [u32],
    pub idx_offset: u64,
}
impl<'a> VertexReserveData<'a> {
    pub fn copy_in(&mut self, vtx: &[Vertex], idx: &[u32]) {
        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);
    }
}
