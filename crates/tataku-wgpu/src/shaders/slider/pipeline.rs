use crate::prelude::*;
use tataku_client_common::prelude::*;

pub fn create_slider_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    projection_matrix_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let slider_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Slider Shader"),
        source: ShaderSource::Wgsl(crate::shader_files::SLIDER.into()),
    });

    let slider_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("slider group layout"),
        entries: &[
            // slider_data
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<SliderData>() as u64 * 2)
                },
                count: None,
            },

            // slider_grids
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<GridCell>() as u64 * 2)
                },
                count: None,
            },

            // grid_cells
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<u32>() as u64 * 2)
                },
                count: None,
            },

            // line_segments
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<LineSegment>() as u64 * 2)
                },
                count: None,
            },

        ],
    });

    let slider_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Slider Pipeline Layout"),
        bind_group_layouts: &[
            projection_matrix_bind_group_layout,
            &slider_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Slider Pipeline"),
        layout: Some(&slider_pipeline_layout),
        cache: None,
        vertex: VertexState {
            module: &slider_shader,
            entry_point: Some("slider_vs_main"),
            buffers: &[ SliderVertex::desc() ],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module: &slider_shader,
            entry_point: Some("slider_fs_main"),
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(WgpuEngine::map_blend_mode(BlendMode::AlphaBlending)),
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
    })
}
