use crate::prelude::*;
use tataku_client_common::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub(crate) const BLURS_PER_BUF:u64 = 1;

pub(crate) struct Buffer {
    pub scissor: Option<Scissor>,
    pub used: bool,
    settings: wgpu::Buffer,
    pub settings_bindgroup: wgpu::BindGroup,
}
impl RenderBufferable for Buffer {
    type Cache = CpuBuffer;
    const VTX_PER_BUF: u64 = BLURS_PER_BUF;
    const IDX_PER_BUF: u64 = BLURS_PER_BUF;

    fn should_write(&self) -> bool { self.used }

    fn reset(&mut self) {
        self.scissor = None;
        self.used = false;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &mut Self::Cache) {
        queue.write_buffer(
            &self.settings, 
            0, 
            bytemuck::cast_slice(&cache.cpu_blurs)
        );
    }

    fn create_new_buffer(device: &wgpu::Device, pipeline: WgpuPipeline) -> Self {
        let settings = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image info"),
            contents: bytemuck::cast_slice(&[super::Params::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let settings_bindgroup = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Compute constants"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: settings.as_entire_binding(),
                    },
                ],
            }
        );

        Self {
            scissor: None,
            used: false,
            settings,
            settings_bindgroup,
        }
    }

}



pub(crate) struct CpuBuffer {
    pub cpu_blurs: Vec<super::Params>,
}
impl Default for CpuBuffer {
    fn default() -> Self {
        Self {
            cpu_blurs: vec![super::Params::default(); BLURS_PER_BUF as usize],
        }
    }
}


pub(crate) struct ReserveData<'a> {
    pub data: &'a mut super::Params,
}
impl ReserveData<'_> {
    pub fn copy_in(
        &mut self, 
        data: super::Params
    ) {
        *self.data = data;
    }
}
