use wgpu::util::{BufferInitDescriptor, DeviceExt};

use crate::prelude::*;
const NAME: &str = "gaussian blur";

pub struct BlurShader {
    pub blur_pipeline: ComputePipeline,
    convert_pipeline: ComputePipeline,
}

impl BlurShader {
    pub fn new(device: &Device) -> Self {
        Self {
            blur_pipeline: Self::gaussian_blur(device),
            convert_pipeline: Self::convert_pipeline(device),
        }
    }

    fn convert_pipeline(device: &Device) -> ComputePipeline {
        const NAME: &str = "convert";
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(format!("{NAME} shader").as_str()),
            source: ShaderSource::Wgsl(crate::shader_files::CONVERT.into()),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(format!("{NAME} pipeline").as_str()),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        pipeline
    }

    fn gaussian_blur(device: &Device) -> ComputePipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(format!("{NAME} shader").as_str()),
            source: ShaderSource::Wgsl(crate::shader_files::BLUR.into()),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(format!("{NAME} pipeline").as_str()),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        pipeline
    }

    fn resize(
        &self,
        device: &Device,
        output: &Texture,
        data: &mut BlurBuffer,
    ) {
        let size = output.size();

        data.vertical.texture = device.create_texture(&TextureDescriptor {
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
        data.horizontal.texture = device.create_texture(&TextureDescriptor {
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

        data.horizontal.bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &self.blur_pipeline.get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &data.vertical.texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &data.horizontal.texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: data.horizontal.buffer.as_entire_binding(),
                },
            ],
        });

        // vertical is updated every draw since it needs the new "swapchain" texture reference
    }

    pub fn perform(
        &self,
        device: &Device,
        queue: &Queue,
        output: &Texture,
        data: &mut BlurBuffer,
    ) {
        let hs = data.horizontal.texture.size();
        if hs != output.size() {
            self.resize(device, output, data);
        }

        data.vertical.bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &self.blur_pipeline.get_bind_group_layout(1),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &output.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &data.vertical.texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: data.vertical.buffer.as_entire_binding(),
                },
            ],
        });


        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                timestamp_writes: None,
                label: Some(format!("{NAME} pass").as_str()),
            });
            compute_pass.set_pipeline(&self.blur_pipeline);
            compute_pass.set_bind_group(0, &data.compute_constants, &[]);
            compute_pass.set_bind_group(1, &data.vertical.bind_group, &[]);
            let (dispatch_width, dispatch_height) = compute_work_group_count(
                (output.width(), output.height()),
                (128, 1),
            );
            compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
            compute_pass.set_bind_group(1, &data.horizontal.bind_group, &[]);
            let (dispatch_height, dispatch_width) = compute_work_group_count(
                (output.width(), output.height()),
                (1, 128),
            );
            compute_pass.dispatch_workgroups(dispatch_width, dispatch_height, 1);
        }

        self.copy_tex_to_tex(
            &mut encoder, 
            device, 
            &data.horizontal.texture, 
            output
        );

        queue.submit(Some(encoder.finish()));
    }


    fn copy_tex_to_tex(
        &self,
        encoder: &mut CommandEncoder,
        device: &Device,
        source: &Texture,
        dest: &Texture,
    ) {

        let size = source.size();
        // need to read texture data then map from rgba8 to bgra8
        let w = size.width;
        let h = size.height;
        let fuck = (w * 4).div_ceil(COPY_BYTES_PER_ROW_ALIGNMENT) * COPY_BYTES_PER_ROW_ALIGNMENT;
        let size = (fuck * h) as u64; //(w * h * 4) as u64;

        let buffer = device.create_buffer(&BufferDescriptor { 
            label: Some("hjkgfdhjklgsd"), 
            size, 
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST | BufferUsages::STORAGE, 
            mapped_at_creation: false
        });

        encoder.copy_texture_to_buffer(
            source.as_image_copy(), 
            ImageCopyBuffer { 
                buffer: &buffer, 
                layout: ImageDataLayout { 
                    offset: 0, 
                    bytes_per_row: Some(fuck), 
                    rows_per_image: None
                }
            }, 
            source.size()
        );

        if false
        {
            let width = source.size().width;

            let buffer2 = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[width]),
                usage: BufferUsages::UNIFORM,
            });

            let mut convert = encoder.begin_compute_pass(&ComputePassDescriptor { label: None, timestamp_writes: None });
            convert.set_pipeline(&self.convert_pipeline);
            let bind_group = device.create_bind_group(&BindGroupDescriptor { 
                label: None, 
                layout: &self.convert_pipeline.get_bind_group_layout(0), 
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: buffer2.as_entire_binding(),
                    },
                ]
            });
            convert.set_bind_group(0, &bind_group, &[]);
            convert.dispatch_workgroups(width, source.size().height, 1);
        }

        encoder.copy_buffer_to_texture(
            ImageCopyBuffer { 
                buffer: &buffer, 
                layout: ImageDataLayout { 
                    offset: 0, 
                    bytes_per_row: Some(fuck), 
                    rows_per_image: None
                }
            }, 
            dest.as_image_copy(), 
            source.size()
        );
    }
}


pub(crate) struct Kernel {
    sum: f32,
    values: Vec<f32>,
}
impl Kernel {
    pub fn new(values: Vec<f32>) -> Self {
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
}

fn kernel_size_for_sigma(sigma: f32) -> u32 {
    2 * (sigma * 3.0).ceil() as u32 + 1
}

pub(super) fn kernel(sigma: f32) -> Kernel {
    let kernel_size = kernel_size_for_sigma(sigma);
    let mut values = vec![0.0; kernel_size as usize];
    let kernel_radius = (kernel_size as usize - 1) / 2;
    for index in 0..=kernel_radius {
        let normpdf = normalized_probablility_density_function(index as f32, sigma);
        values[kernel_radius + index] = normpdf;
        values[kernel_radius - index] = normpdf;
    }

    Kernel::new(values)
}

fn normalized_probablility_density_function(x: f32, sigma: f32) -> f32 {
    0.39894 * (-0.5 * x * x / (sigma * sigma)).exp() / sigma
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
pub(crate) fn compute_work_group_count(
    (width, height): (u32, u32),
    (workgroup_width, workgroup_height): (u32, u32),
) -> (u32, u32) {
    let width = width.div_ceil(workgroup_width);
    let height = height.div_ceil(workgroup_height);

    (width, height)
}

