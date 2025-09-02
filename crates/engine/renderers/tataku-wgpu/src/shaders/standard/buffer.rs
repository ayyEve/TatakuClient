use crate::prelude::*;
use crate::wgpu_engine::WgpuPipeline;
use crate::buffer_queue::RenderBufferable;

const QUAD_PER_BUF:u64 = 3000;

pub(crate) struct Buffer {
    pub blend_mode: tataku::GraphicsPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub scissor: Option<tataku::Scissor>,

    pub used_vertices: u64,
    pub used_indices: u64,
}
impl RenderBufferable for Buffer {
    type Cache = CpuBuffer;
    const VTX_PER_BUF: u64 = QUAD_PER_BUF * 4;
    const IDX_PER_BUF: u64 = QUAD_PER_BUF * 6;
    
    // fn name() -> &'static str { "vertex buffer" }
    fn should_write(&self) -> bool { self.used_indices > 0 }

    fn reset(&mut self) {
        self.blend_mode = tataku::GraphicsPipeline::None;
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &mut Self::Cache) {
        queue.write_buffer(
            &self.vertex_buffer, 
            0, 
            bytemuck::cast_slice(&cache.cpu_vtx)
        );
        queue.write_buffer(
            &self.index_buffer, 
            0, 
            bytemuck::cast_slice(&cache.cpu_idx)
        );
    }

    fn create_new_buffer(device: &wgpu::Device, _: WgpuPipeline) -> Self {
        Self {
            blend_mode: tataku::GraphicsPipeline::None,
            scissor: None,
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: Self::VTX_PER_BUF * size_of::<super::Vertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size: Self::IDX_PER_BUF * size_of::<u32>() as u64,
                mapped_at_creation: false,
            }),
            used_vertices: 0,
            used_indices: 0,
        }
    }
}

pub(crate) struct CpuBuffer {
    pub cpu_vtx: Vec<super::Vertex>,
    pub cpu_idx: Vec<u32>,
}
impl Default for CpuBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![
                super::Vertex::default(); 
                super::Buffer::VTX_PER_BUF as usize
            ],
            cpu_idx: vec![0; super::Buffer::IDX_PER_BUF as usize],
        }
    }
}


pub(crate) struct ReserveData<'a> {
    pub vtx: &'a mut [super::Vertex],
    pub idx: &'a mut [u32],
    pub idx_offset: u64,
}
impl ReserveData<'_> {
    pub fn copy_in(&mut self, vtx: &[super::Vertex], idx: &[u32]) {
        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);
    }
}
