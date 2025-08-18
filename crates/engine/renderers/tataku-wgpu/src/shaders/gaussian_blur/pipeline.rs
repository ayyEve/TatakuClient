use crate::prelude::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const NAME: &str = "gaussian blur";

pub struct GaussianBlurShader {
    pub pipeline: ComputePipeline,

    vertical: BlurBindings,
    horizontal: BlurBindings,
}
impl GaussianBlurShader {
    pub fn new(device: &Device) -> Self {
        let pipeline = Self::gaussian_blur(device);

        // some default size, will get updated later
        let size = Extent3d { 
            width: 1, 
            height: 1, 
            depth_or_array_layers: 1 
        };

        let vertical_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Orientation"),
                contents: bytemuck::cast_slice(&[1u32]),
                usage: BufferUsages::UNIFORM,
            }
        );
        let horizontal_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Orientation"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: BufferUsages::UNIFORM,
            }
        );


        let desc = TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        };

        let vertical_texture = device.create_texture(&desc);
        let horizontal_texture = device.create_texture(&desc);

        let vertical_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            // NOTE!: this should be the output texture, but thats not accessible here
                            &vertical_texture.create_view(
                                &TextureViewDescriptor::default()
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(
                            &vertical_texture.create_view(
                                &TextureViewDescriptor::default()
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: vertical_buffer.as_entire_binding(),
                    },
                ],
            }
        );

        let horizontal_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            &vertical_texture.create_view(
                                &TextureViewDescriptor::default()
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(
                            &horizontal_texture.create_view(
                                &TextureViewDescriptor::default()
                            ),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: horizontal_buffer.as_entire_binding(),
                    },
                ],
            }
        );

        Self {
            pipeline,

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

    fn gaussian_blur(device: &Device) -> ComputePipeline {
        let shader = device.create_shader_module(
            ShaderModuleDescriptor {
                label: Some(format!("{NAME} shader").as_str()),
                source: ShaderSource::Wgsl(crate::shader_files::GAUSSIAN_BLUR.into()),
            }
        );

        device.create_compute_pipeline(
            &ComputePipelineDescriptor {
                label: Some(format!("{NAME} pipeline").as_str()),
                layout: None,
                module: &shader,
                entry_point: Some("main"),
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            }
        )
    }

    fn resize(
        &mut self,
        device: &Device,
        output: &WgpuTextureReference,
    ) {
        let desc = TextureDescriptor {
            label: None,
            size: output.size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        };


        self.vertical.texture = device.create_texture(&desc);
        self.horizontal.texture = device.create_texture(&desc);

        let view_desc = TextureViewDescriptor::default();

        self.vertical.bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &self.pipeline.get_bind_group_layout(1),
                entries: &[
                    // input
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&output.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(
                            &self.vertical.texture.create_view(&view_desc),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: self.vertical.buffer.as_entire_binding(),
                    },
                ],
            }
        );

        self.horizontal.bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &self.pipeline.get_bind_group_layout(1),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(
                            &self.vertical.texture.create_view(&view_desc),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(
                            &self.horizontal.texture.create_view(&view_desc),
                        ),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: self.horizontal.buffer.as_entire_binding(),
                    },
                ],
            }
        );

    }

    pub fn perform(
        &mut self,
        device: &Device,
        queue: &Queue,
        output: &WgpuTextureReference,
        data: &GaussianBlurBuffer,
    ) {
        let hs = self.horizontal.texture.size();
        if hs != output.size {
            self.resize(device, output);
        }

        // perform the blur
        let mut encoder = device.create_command_encoder(
            &CommandEncoderDescriptor { label: None }
        );
        {
            let mut compute_pass = encoder.begin_compute_pass(
                &ComputePassDescriptor {
                    timestamp_writes: None,
                    label: Some(format!("{NAME} pass").as_str()),
                }
            );
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(
                0, 
                &data.compute_constants, 
                &[]
            );
            compute_pass.set_bind_group(
                1, 
                &self.vertical.bind_group, 
                &[]
            );

            let (dispatch_width, dispatch_height) = compute_work_group_count(
                (output.size.width, output.size.height),
                (16, 16),
            );
            compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
            compute_pass.set_bind_group(
                1, 
                &self.horizontal.bind_group, 
                &[]
            );

            let (dispatch_width, dispatch_height) = compute_work_group_count(
                (output.size.width, output.size.height),
                (16, 16),
            );
            compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
        }

        encoder.copy_texture_to_texture(
            self.horizontal.texture.as_image_copy(), 
            output.copy, 
            output.size,
        );

        queue.submit(Some(encoder.finish()));
    }

}


pub struct GaussianKernel {
    sum: f32,
    values: Vec<f32>,
}
impl GaussianKernel {
    fn new(values: Vec<f32>) -> Self {
        let sum = values.iter().sum();
        Self { sum, values }
    }

    pub fn packed_data(&self) -> Vec<f32> {
        let mut data = vec![0.0; self.values.len() + 1];
        data[0] = self.sum;
        data[1..].copy_from_slice(&self.values);
        data
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn kernel(sigma: f32) -> GaussianKernel {
        let kernel_size = kernel_size_for_sigma(sigma);
        let mut values = vec![0.0; kernel_size as usize];
        let kernel_radius = (kernel_size as usize - 1) / 2;
        for index in 0..=kernel_radius {
            let normpdf = normalized_probablility_density_function(
                index as f32, 
                sigma
            );
            values[kernel_radius + index] = normpdf;
            values[kernel_radius - index] = normpdf;
        }

        GaussianKernel::new(values)
    }
}

/// Calculate using pixels within 3 sigma
fn kernel_size_for_sigma(sigma: f32) -> u32 {
    2 * (sigma * 3.0).ceil() as u32 + 1
}

fn normalized_probablility_density_function(x: f32, sigma: f32) -> f32 {
    0.39894 * (-0.5 * x * x / (sigma * sigma)).exp() / sigma
}



struct BlurBindings {
    buffer: Buffer,
    texture: Texture,
    bind_group: BindGroup,
}





/// Compute the amount of work groups to be dispatched for an image, based on the work group size.
/// Chances are, the group will not match perfectly, like an image of width 100, for a workgroup size of 32.
/// To make sure the that the whole 100 pixels are visited, then we would need a count of 4, as 4 * 32 = 128,
/// which is bigger than 100. A count of 3 would be too little, as it means 96, so four columns (or, 100 - 96) would be ignored.
///
/// # Arguments
///
/// * `(width, height)` - The dimension of the image we are working on.
/// * `(workgroup_width, workgroup_height)` - The width and height dimensions of the compute workgroup.
fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let width = width.div_ceil(workgroup_width);
    let height = height.div_ceil(workgroup_height);

    (width, height)
}
