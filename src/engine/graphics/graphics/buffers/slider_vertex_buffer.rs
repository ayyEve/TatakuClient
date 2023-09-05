// use super::super::Scissor;
use wgpu::{
    Buffer,
    Device
};
use crate::prelude::*;

const QUAD_PER_BUF:u64 = 3000;
const VTX_PER_BUF:u64 = QUAD_PER_BUF * 4;
const IDX_PER_BUF:u64 = QUAD_PER_BUF * 6;

pub struct SliderVertexBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub scissor: Option<Scissor>,

    // pub scissor_buffer: Buffer,
    // pub scissor_buffer_bind_group: BindGroup,

    pub used_vertices: u64,
    pub used_indices: u64,
    // pub used_scissors: u64,

    // recording_periods_since_last_use: usize
}

impl RenderBufferable for SliderVertexBuffer {
    type Cache = CpuSliderVertexBuffer;

    fn reset(&mut self) {
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&cache.cpu_vtx));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&cache.cpu_idx));
    }

    fn should_write(&self) -> bool {
        self.used_indices > 0
    }

    fn create_new_buffer(device: &Device) -> Self {
        SliderVertexBuffer {
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

pub struct CpuSliderVertexBuffer {
    pub cpu_vtx: Vec<SliderVertex>,
    pub cpu_idx: Vec<u32>,
}
impl Default for CpuSliderVertexBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![SliderVertex::default(); VTX_PER_BUF as usize],
            cpu_idx: vec![0; IDX_PER_BUF as usize],
        }
    }
}


pub struct SliderVertexReserveData<'a> {
    pub vtx: &'a mut [SliderVertex],
    pub idx: &'a mut [u32],
    pub idx_offset: u64,
}
impl<'a> SliderVertexReserveData<'a> {
    pub fn copy_in(&mut self, vtx: &[SliderVertex], idx: &[u32]) {
        // std::mem::swap(vtx, self.vtx);
        // std::mem::swap(idx, self.idx);
        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);

        // for i in 0..vtx.len() { self.vtx[i] = vtx[i] }
        // for i in 0..idx.len() { self.idx[i] = idx[i] }
    }
}
