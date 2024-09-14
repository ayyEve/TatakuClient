use wgpu::{
    BindGroup,
    Buffer,
    Device
};

use crate::prelude::*;
use tokio::sync::OnceCell;
use tataku_client_common::prelude::*;


pub static FLASHLIGHT_BIND_GROUP_LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::const_new();

const FLASHLIGHT_PER_BUF: u64 = 4; // even if we're drawing multiple flashlights, they wont be drawn consecutively
const VTX_PER_BUF:u64 = FLASHLIGHT_PER_BUF * 4;
const IDX_PER_BUF:u64 = FLASHLIGHT_PER_BUF * 6;

pub struct FlashlightBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub flashlight_buffer: Buffer,
    pub scissor: Option<Scissor>,
    pub bind_group: BindGroup,

    pub used_vertices: u64,
    pub used_indices: u64,
    pub used_flashlights: u64,
}

impl RenderBufferable for FlashlightBuffer {
    type Cache = CpuFlashlightBuffer;
    const VTX_PER_BUF: u64 = VTX_PER_BUF;
    const IDX_PER_BUF: u64 = IDX_PER_BUF;

    // fn name() -> &'static str { "Flashlight buffer" }
    fn should_write(&self) -> bool { self.used_flashlights > 0 }

    fn reset(&mut self) {
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;
        self.used_flashlights = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&cache.cpu_vtx));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&cache.cpu_idx));
        queue.write_buffer(&self.flashlight_buffer, 0, bytemuck::cast_slice(&cache.cpu_flashlights));
    }

    fn create_new_buffer(device: &Device) -> Self {
        let bind_group_layout = FLASHLIGHT_BIND_GROUP_LAYOUT.get().unwrap();

        let flashlight_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Flashlight Data Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: FLASHLIGHT_PER_BUF * std::mem::size_of::<FlashlightDataInner>() as u64,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("flashlight bind group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: flashlight_buffer.as_entire_binding() },
            ]
        });

        Self {
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Flashlight Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: VTX_PER_BUF * std::mem::size_of::<FlashlightVertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Flashlight Index Buffer"),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size: IDX_PER_BUF * std::mem::size_of::<u32>() as u64,
                mapped_at_creation: false,
            }),
            flashlight_buffer,
            bind_group,
            
            scissor: None,
            used_vertices: 0,
            used_indices: 0,
            used_flashlights: 0,
        }
    }


}

pub struct CpuFlashlightBuffer {
    pub cpu_vtx: Vec<FlashlightVertex>,
    pub cpu_idx: Vec<u32>,
    pub cpu_flashlights: Vec<FlashlightDataInner>,
}
impl Default for CpuFlashlightBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![FlashlightVertex::default(); VTX_PER_BUF as usize],
            cpu_idx: vec![0; IDX_PER_BUF as usize],
            cpu_flashlights: vec![FlashlightDataInner::default(); FLASHLIGHT_PER_BUF as usize],
        }
    }
}


pub struct FlashlightReserveData<'a> {
    pub vtx: &'a mut [FlashlightVertex],
    pub idx: &'a mut [u32],
    pub flashlight_data: &'a mut FlashlightDataInner,

    pub idx_offset: u64,
    pub flashlight_index: u32
}
impl<'a> FlashlightReserveData<'a> {
    pub fn copy_in(&mut self, vtx: &[FlashlightVertex], flashlight_data: FlashlightData) {
        let offset = self.idx_offset as u32;
        let idx:&[u32] = &[
            offset,
            2 + offset,
            1 + offset,

            1 + offset,
            2 + offset,
            3 + offset,
        ];

        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);
        *self.flashlight_data = flashlight_data.into();
    }
}
