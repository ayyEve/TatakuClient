use crate::prelude::*;

const FLASHLIGHT_PER_BUF: u64 = 4; // even if we're drawing multiple flashlights, they wont be drawn consecutively
const VTX_PER_BUF:u64 = FLASHLIGHT_PER_BUF * 4;
const IDX_PER_BUF:u64 = FLASHLIGHT_PER_BUF * 6;

pub(crate) struct Buffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub flashlight_buffer: wgpu::Buffer,
    pub scissor: Option<tataku::Scissor>,
    pub bind_group: wgpu::BindGroup,

    pub used_vertices: u64,
    pub used_indices: u64,
    pub used_flashlights: u64,
}
impl RenderBufferable for Buffer {
    type Cache = CpuBuffer;
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
        queue.write_buffer(
            &self.flashlight_buffer, 
            0, 
            bytemuck::cast_slice(&cache.cpu_flashlights)
        );
    }

    fn create_new_buffer(device: &wgpu::Device, pipeline: WgpuPipeline) -> Self {
        let flashlight_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Flashlight Data Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: FLASHLIGHT_PER_BUF * size_of::<super::GpuData>() as u64,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("flashlight bind group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry { 
                        binding: 0, 
                        resource: flashlight_buffer.as_entire_binding() 
                    },
                ]
            }
        );

        Self {
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Flashlight Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: VTX_PER_BUF * size_of::<super::Vertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Flashlight Index Buffer"),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size: IDX_PER_BUF * size_of::<u32>() as u64,
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

pub(crate) struct CpuBuffer {
    pub cpu_vtx: Vec<super::Vertex>,
    pub cpu_idx: Vec<u32>,
    pub cpu_flashlights: Vec<super::GpuData>,
}
impl Default for CpuBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![super::Vertex::default(); VTX_PER_BUF as usize],
            cpu_idx: vec![0; IDX_PER_BUF as usize],
            cpu_flashlights: vec![super::GpuData::default(); FLASHLIGHT_PER_BUF as usize],
        }
    }
}


pub(crate) struct ReserveData<'a> {
    pub vtx: &'a mut [super::Vertex],
    pub idx: &'a mut [u32],
    pub flashlight_data: &'a mut super::GpuData,

    pub idx_offset: u64,
    pub flashlight_index: u32
}
impl ReserveData<'_> {
    pub fn copy_in(
        &mut self, 
        vtx: &[super::Vertex], 
        flashlight_data: tataku::FlashlightData
    ) {
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
