

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature="graphics", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct FlashlightData {
    pub cursor_pos: [f32; 2],
    pub flashlight_radius: f32,
    pub fade_radius: f32,
    pub color: [f32; 4]
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature="graphics", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct FlashlightVertex {
    pub position: [f32; 2],
    pub flashlight_index: u32,
}
#[cfg(feature="graphics")]
impl FlashlightVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // todo: convert to macro

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // flashlight index
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
            ]
        }
    }
}



#[cfg(feature="graphics")]
pub fn create_flashlight_pipeline(
    device: &wgpu::Device, 
    config: &wgpu::SurfaceConfiguration,
    projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    use super::FLASHLIGHT_BIND_GROUP_LAYOUT;


    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Flashlight Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/flashlight.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("flashlight group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<FlashlightData>() as u64 * 2)
                },
                count: None,
            },
        ],
    });
    FLASHLIGHT_BIND_GROUP_LAYOUT.set(bind_group_layout).unwrap();

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Flashlight Pipeline Layout"),
        bind_group_layouts: &[
            &projection_matrix_bind_group_layout,
            FLASHLIGHT_BIND_GROUP_LAYOUT.get().unwrap(),
        ],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Flashlight Pipeline")),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "flashlight_vs_main",
            buffers: &[ FlashlightVertex::desc() ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "flashlight_fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(crate::prelude::BlendMode::AlphaBlending.get_blend_state()),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    pipeline
}

