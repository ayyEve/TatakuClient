use crate::prelude::*;
use tataku_client_common::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use super::kernel;
pub const BLURS_PER_BUF:u64 = 1;

pub struct BlurBuffer {
    pub scissor: Option<Scissor>,
    pub used: u64,
    kernel_size: u32,
    sigma: f32,
    settings: Buffer,
    kernel_buffer: Buffer,

    pub vertical: BlurBindings,
    pub horizontal: BlurBindings,
    pub compute_constants: BindGroup,
}

pub struct BlurBindings {
    pub buffer: Buffer,
    pub texture: Texture,
    pub bind_group: BindGroup,
}

impl RenderBufferable for BlurBuffer {
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
            let kernel = kernel(params.sigma);
            self.kernel_size = kernel.size() as u32;
            queue.write_buffer(&self.kernel_buffer, 0, bytemuck::cast_slice(&kernel.packed_data()[..]));
        }

        let settings = Blur2 {
            filter_size: self.kernel_size,
            x: params.x,
            y: params.y,
            width: params.width,
            height: params.height,
        };

        queue.write_buffer(&self.settings, 0, bytemuck::cast_slice(&[settings]));
    }

    fn create_new_buffer(device: &Device, pipeline: WgpuPipeline) -> Self {
        // some default size, will get updated later
        let size = Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
        let sigma = 100.0; // this affects the size of the buffer, so we start with an unreasonably high number to hopefully prevent crashes when its changed later

        let kernel = kernel(sigma);
        let kernel_size = kernel.size() as u32;

        let settings = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Image info"),
            contents: bytemuck::cast_slice(&[Blur2::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let kernel = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&kernel.packed_data()[..]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let compute_constants = device.create_bind_group(&BindGroupDescriptor {
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
        });


        let vertical_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Orientation"),
            contents: bytemuck::cast_slice(&[1u32]),
            usage: BufferUsages::UNIFORM,
        });
        let horizontal_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Orientation"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: BufferUsages::UNIFORM,
        });


        let vertical_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let horizontal_texture = device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let vertical_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        // NOTE!: this should be the output texture, but thats not accessible here
                        &vertical_texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &vertical_texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: vertical_buffer.as_entire_binding(),
                },
            ],
        });

        let horizontal_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &vertical_texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &horizontal_texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: horizontal_buffer.as_entire_binding(),
                },
            ],
        });


        Self {
            scissor: None,
            used: 0,
            sigma,
            kernel_size,
            settings,
            kernel_buffer: kernel,
            compute_constants,

            horizontal: BlurBindings {
                buffer: horizontal_buffer,
                texture: horizontal_texture,
                bind_group: horizontal_bind_group
            },
            vertical: BlurBindings {
                buffer: vertical_buffer,
                texture: vertical_texture,
                bind_group: vertical_bind_group
            },
        }
    }


}



pub struct CpuBlurBuffer {
    // pub cpu_vtx: Vec<BlurVertex>,
    // pub cpu_idx: Vec<u32>,
    pub cpu_blurs: Vec<BlurParams>,
}
impl Default for CpuBlurBuffer {
    fn default() -> Self {
        Self {
            // cpu_vtx: vec![FlashlightVertex::default(); VTX_PER_BUF as usize],
            // cpu_idx: vec![0; 1 as usize],
            cpu_blurs: vec![BlurParams::default(); BLURS_PER_BUF as usize],
        }
    }
}


pub struct BlurReserveData<'a> {
    pub data: &'a mut BlurParams,

    // pub idx_offset: u64,
    pub _blur_index: u32,
}
impl BlurReserveData<'_> {
    pub fn copy_in(
        &mut self, 
        // vtx: &[BlurVertex], 
        data: BlurParams
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