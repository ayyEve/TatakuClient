use crate::prelude::*;
use tataku_client_common::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub const BLURS_PER_BUF:u64 = 1;

pub struct BoxBlurBuffer {
    pub scissor: Option<Scissor>,
    pub used: bool,
    settings: Buffer,
    pub settings_bindgroup: BindGroup,
}

impl RenderBufferable for BoxBlurBuffer {
    type Cache = CpuBoxBlurBuffer;
    const VTX_PER_BUF: u64 = BLURS_PER_BUF;
    const IDX_PER_BUF: u64 = BLURS_PER_BUF;

    fn should_write(&self) -> bool { self.used }

    fn reset(&mut self) {
        self.scissor = None;
        self.used = false;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
        queue.write_buffer(
            &self.settings, 
            0, 
            bytemuck::cast_slice(&cache.cpu_blurs)
        );
    }

    fn create_new_buffer(device: &Device, pipeline: WgpuPipeline) -> Self {
        let settings = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image info"),
            contents: bytemuck::cast_slice(&[BoxBlurParams::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let settings_bindgroup = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Compute constants"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
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



pub struct CpuBoxBlurBuffer {
    pub cpu_blurs: Vec<BoxBlurParams>,
}
impl Default for CpuBoxBlurBuffer {
    fn default() -> Self {
        Self {
            cpu_blurs: vec![BoxBlurParams::default(); BLURS_PER_BUF as usize],
        }
    }
}


pub struct BoxBlurReserveData<'a> {
    pub data: &'a mut BoxBlurParams,
}
impl BoxBlurReserveData<'_> {
    pub fn copy_in(
        &mut self, 
        data: BoxBlurParams
    ) {
        *self.data = data;
    }
}
