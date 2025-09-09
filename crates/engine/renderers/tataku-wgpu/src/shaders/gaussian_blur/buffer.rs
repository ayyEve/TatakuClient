use crate::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const BLURS_PER_BUF:u64 = 1;

pub(crate) struct Buffer {
    pub scissor: Option<tataku::Scissor>,
    pub used: u64,
    kernel_size: u32,
    sigma: f32,
    
    settings: wgpu::Buffer,
    kernel_buffer: wgpu::Buffer,
    pub compute_constants: wgpu::BindGroup,
}
impl RenderBufferable for Buffer {
    type Cache = CpuBuffer;
    const VTX_PER_BUF: u64 = BLURS_PER_BUF;
    const IDX_PER_BUF: u64 = BLURS_PER_BUF;

    fn should_write(&self) -> bool { self.used > 0 }

    fn reset(&mut self) {
        self.scissor = None;
        self.used = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &mut Self::Cache) {
        let params = cache.cpu_blurs[0];

        if self.sigma != params.sigma {
            self.sigma = params.sigma;
            let kernel = super::GaussianKernel::kernel(params.sigma);
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

    fn create_new_buffer(device: &wgpu::Device, pipeline: WgpuPipeline) -> Self {
        let sigma = 100.0; // this affects the size of the buffer, so we start with an unreasonably high number to hopefully prevent crashes when its changed later

        let kernel = super::GaussianKernel::kernel(sigma);
        let kernel_size = kernel.size() as u32;

        let settings = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image info"),
            contents: bytemuck::cast_slice(&[Blur2::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let kernel = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("gaussian blur"),
                contents: bytemuck::cast_slice(&kernel.packed_data()[..]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let compute_constants = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Compute constants"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: settings.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
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
    pub _blur_index: u32,
}
impl ReserveData<'_> {
    pub fn copy_in(
        &mut self,
        data: super::Params
    ) {
        *self.data = data;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct Blur2 {
    filter_size: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
