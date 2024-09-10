
use crate::prelude::*;
use tataku_client_common::prelude::*;

pub fn create_slider_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {

    let slider_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Slider Shader"),
        source: wgpu::ShaderSource::Wgsl(tataku_resources::shaders::SLIDER.into()),
    });

    let slider_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("slider group layout"),
        entries: &[
            // slider_data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<SliderData>() as u64 * 2)
                },
                count: None,
            },

            // slider_grids
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<GridCell>() as u64 * 2)
                },
                count: None,
            },

            // grid_cells
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<u32>() as u64 * 2)
                },
                count: None,
            },

            // line_segments
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<LineSegment>() as u64 * 2)
                },
                count: None,
            },

        ],
    });
    SLIDER_BIND_GROUP_LAYOUT.set(slider_bind_group_layout).unwrap();

    let slider_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Slider Pipeline Layout"),
        bind_group_layouts: &[
            &projection_matrix_bind_group_layout,
            SLIDER_BIND_GROUP_LAYOUT.get().unwrap(),
        ],
        push_constant_ranges: &[],
    });

    let slider_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Slider Pipeline")),
        layout: Some(&slider_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &slider_shader,
            entry_point: "slider_vs_main",
            buffers: &[ SliderVertex::desc() ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &slider_shader,
            entry_point: "slider_fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(WgpuEngine::map_blend_mode(BlendMode::AlphaBlending)),
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

    slider_pipeline
}
