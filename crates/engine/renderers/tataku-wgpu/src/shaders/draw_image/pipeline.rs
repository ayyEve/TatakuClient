use crate::prelude::*;
use tataku_client_common::graphics::Pipeline;

pub struct RenderImageShader {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub buffer: Buffer,
}
impl RenderImageShader {
    pub fn new(
        device: &Device,
        queue: &Queue,
        projection_matrix_layout: &BindGroupLayout,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None
                    },
                ]
            }
        );

        let shader = device.create_shader_module(
            ShaderModuleDescriptor {
                label: Some("nvjdks"),
                source: ShaderSource::Wgsl(crate::shader_files::RENDER_IMAGE.into()),
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: Some("hgvnfjkdsmlc"),
                bind_group_layouts: &[
                    &bind_group_layout,
                    projection_matrix_layout
                ],
                push_constant_ranges: &[]
            }
        );

        let pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor {
            label: Some("vhfjkdnhijvds"),
            cache: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: size_of::<[f32; 2]>() as u64,
                                shader_location: 1
                            },
                        ]
                    }
                ]
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8UnormSrgb,
                    blend: Some(WgpuEngine::map_blend_mode(Pipeline::AlphaBlending)),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let buffer = device.create_buffer(&BufferDescriptor { 
            label: Some("nhjgkdnkvdskmdvs"), 
            size: size_of::<[Vertex; 6]>() as u64, 
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST, 
            mapped_at_creation: false
        });

        let tl = Vertex::new([0.0, 0.0], [0.0, 0.0]);
        let tr = Vertex::new([1.0, 0.0], [1.0, 0.0]);
        let bl = Vertex::new([0.0, 1.0], [0.0, 1.0]);
        let br = Vertex::new([1.0, 1.0], [1.0, 1.0]);

        let data = [
            tl, bl, tr,
            tr, bl, br,
        ];

        if let Some(mut a) = queue.write_buffer_with(
            &buffer, 0, 
            NonZero::new(size_of::<[Vertex; 6]>() as u64).unwrap()
        ) {
            a.as_mut().copy_from_slice(bytemuck::cast_slice(&data)); 
        }
        queue.submit([]);

        Self {
            pipeline,
            bind_group_layout,
            buffer
        }
    }

    pub fn update_buffer(
        &self, 
        queue: &Queue, 
        [width, height]: [u32; 2]
    ) {
        let tl = Vertex::new([0.0, 0.0], [0.0, 0.0]);
        let tr = Vertex::new([width as f32, 0.0], [1.0, 0.0]);
        let bl = Vertex::new([0.0, height as f32], [0.0, 1.0]);
        let br = Vertex::new([width as f32, height as f32], [1.0, 1.0]);
        let data = [
            tl, bl, tr,
            tr, bl, br,
        ];

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&data));
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_uv: [f32; 2]
}
impl Vertex {
    fn new(
        p: impl Into<[f32; 2]>, 
        t: impl Into<[f32; 2]>
    ) -> Self {
        Self {
            position: p.into(),
            tex_uv: t.into(),
        }
    }
}
