use crate::prelude::*;
use tataku_client_common::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const BLURS_PER_BUF:u64 = 1;

pub struct GaussianBlurBuffer {
    pub scissor: Option<Scissor>,
    pub used: u64,
    kernel_size: u32,
    sigma: f32,
    settings: Buffer,
    kernel_buffer: Buffer,

    pub compute_constants: BindGroup,
}

impl RenderBufferable for GaussianBlurBuffer {
    type Cache = CpuBlurBuffer;
    const VTX_PER_BUF: u64 = BLURS_PER_BUF;
    const IDX_PER_BUF: u64 = BLURS_PER_BUF;

    fn should_write(&self) -> bool { self.used > 0 }

    fn reset(&mut self) {
        self.scissor = None;
        self.used = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
        let params = cache.cpu_blurs[0];
        
        if self.sigma != params.sigma {
            self.sigma = params.sigma;
            let kernel = GaussianKernel::kernel(params.sigma);
            self.kernel_size = kernel.size() as u32;
            queue.write_buffer(
                &self.kernel_buffer, 
                0, 
                bytemuck::cast_slice(&kernel.packed_data()[..])
            );
        }

        let settings = Blur2 {
            filter_size: self.kernel_size,
            x: params.x,
            y: params.y,
            width: params.width,
            height: params.height,
        };

        queue.write_buffer(
            &self.settings, 
            0, 
            bytemuck::cast_slice(&[settings])
        );
    }

    fn create_new_buffer(device: &Device, pipeline: WgpuPipeline) -> Self {
        let sigma = 100.0; // this affects the size of the buffer, so we start with an unreasonably high number to hopefully prevent crashes when its changed later

            let kernel = GaussianKernel::kernel(sigma);
        let kernel_size = kernel.size() as u32;

        let settings = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image info"),
            contents: bytemuck::cast_slice(&[Blur2::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let kernel = device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&kernel.packed_data()[..]),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            }
        );

        let compute_constants = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Compute constants"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: settings.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: kernel.as_entire_binding(),
                    },
                ],
            }
        );

        Self {
            scissor: None,
            used: 0,
            sigma,
            kernel_size,
            settings,
            kernel_buffer: kernel,
            compute_constants,
        }
    }

}



pub struct CpuBlurBuffer {
    pub cpu_blurs: Vec<GaussianBlurParams>,
}
impl Default for CpuBlurBuffer {
    fn default() -> Self {
        Self {
            cpu_blurs: vec![GaussianBlurParams::default(); BLURS_PER_BUF as usize],
        }
    }
}


pub struct GaussianBlurReserveData<'a> {
    pub data: &'a mut GaussianBlurParams,
    pub _blur_index: u32,
}
impl GaussianBlurReserveData<'_> {
    pub fn copy_in(
        &mut self, 
        data: GaussianBlurParams
    ) {
        *self.data = data;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct Blur2 {
    filter_size: u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}
