use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::renderable_surface::WgpuTextureReference;

const NAME: &str = "box blur";

pub(crate) struct Pipeline {
    pub pipeline: wgpu::ComputePipeline,

    vertical: BlurBindings,
    horizontal: BlurBindings,
}
impl Pipeline {
    pub fn new(device: &wgpu::Device, output: &WgpuTextureReference) -> Self {
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some(format!("{NAME} shader").as_str()),
                source: wgpu::ShaderSource::Wgsl(crate::shader_files::BOX_BLUR.into()),
            }
        );

        let pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some(format!("{NAME} pipeline").as_str()),
                layout: None,
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            }
        );

        let vertical_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Orientation"),
                contents: bytemuck::cast_slice(&[1u32]),
                usage: wgpu::BufferUsages::UNIFORM,
            }
        );
        let horizontal_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Orientation"),
                contents: bytemuck::cast_slice(&[0u32]),
                usage: wgpu::BufferUsages::UNIFORM,
            }
        );

        let [
            vertical, 
            horizontal
        ] = Self::create_textures(
            device, 
            output, 
            &pipeline, 
            &vertical_buffer,
            &horizontal_buffer,
        );

        Self {
            pipeline,

            horizontal: BlurBindings {
                buffer: horizontal_buffer,
                texture: horizontal.0,
                bind_group: horizontal.1,
            },
            vertical: BlurBindings {
                buffer: vertical_buffer,
                texture: vertical.0,
                bind_group: vertical.1,
            },
        }
    }

    fn create_textures(
        device: &wgpu::Device,
        output: &WgpuTextureReference,
        pipeline: &wgpu::ComputePipeline,
        vertical_buffer: &wgpu::Buffer,
        horizontal_buffer: &wgpu::Buffer,
    ) -> [(wgpu::Texture, wgpu::BindGroup); 2] {
        let format = output.view
            .texture()
            .format()
            .remove_srgb_suffix();

        let desc = wgpu::TextureDescriptor {
            label: Some(NAME),
            size: output.size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[ format.add_srgb_suffix() ],
        };

        let view_desc = wgpu::TextureViewDescriptor::default();

        let vertical_texture = device.create_texture(&desc);
        let horizontal_texture = device.create_texture(&desc);

        let vertical_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &output.view,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &vertical_texture.create_view(&view_desc),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            vertical_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            }
        );

        let horizontal_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Texture bind group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &vertical_texture.create_view(&view_desc),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &horizontal_texture.create_view(&view_desc),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            horizontal_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            }
        );

        [
            (vertical_texture, vertical_bind_group),
            (horizontal_texture, horizontal_bind_group),
        ]
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        output: &WgpuTextureReference,
    ) {
        let [
            vertical, 
            horizontal
        ] = Self::create_textures(
            device, 
            output, 
            &self.pipeline, 
            &self.vertical.buffer,
            &self.horizontal.buffer,
        );

        self.vertical.texture = vertical.0;
        self.vertical.bind_group = vertical.1;

        self.horizontal.texture = horizontal.0;
        self.horizontal.bind_group = horizontal.1;
    }

    pub fn perform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output: &WgpuTextureReference,
        data: &super::Buffer,
    ) {
        if self.horizontal.texture.size() != output.size {
            self.resize(device, output);
        }

        // perform the blur
        let mut encoder = device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some(NAME) }
        );
        {
            let mut compute_pass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor {
                    timestamp_writes: None,
                    label: Some(format!("{NAME} pass").as_str()),
                }
            );

            let (x, y) = compute_work_group_count(
                (output.size.width, output.size.height),
                (16, 16),
            );
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(
                0,
                &data.settings_bindgroup,
                &[]
            );
            
            // vertical first
            compute_pass.set_bind_group(
                1,
                &self.vertical.bind_group,
                &[]
            );
            compute_pass.dispatch_workgroups(x, y, 1);

            // horizontal after
            compute_pass.set_bind_group(
                1,
                &self.horizontal.bind_group,
                &[]
            );
            compute_pass.dispatch_workgroups(x, y, 1);
        }


        // copy the blur to the output
        wgpu::util::TextureBlitter::new(
            device, 
            output.view.texture().format(),
        ).copy(
            device,
            &mut encoder,
            &self.horizontal.texture.create_view(
                &wgpu::TextureViewDescriptor::default()
            ),
            &output.view
        );

        // encoder.copy_texture_to_texture(
        //     self.horizontal.texture.as_image_copy(),
        //     output.copy,
        //     output.size,
        // );

        queue.submit(Some(encoder.finish()));
    }

}


struct BlurBindings {
    buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
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
