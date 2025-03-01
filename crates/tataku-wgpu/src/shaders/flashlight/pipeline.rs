use tataku_client_common::prelude::*;
use crate::prelude::*;

pub fn create_flashlight_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    use super::FLASHLIGHT_BIND_GROUP_LAYOUT;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Flashlight Shader"),
        source: wgpu::ShaderSource::Wgsl(crate::shader_files::FLASHLIGHT.into()),
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
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<FlashlightDataInner>() as u64 * 2)
                },
                count: None,
            },
        ],
    });
    FLASHLIGHT_BIND_GROUP_LAYOUT.set(bind_group_layout).unwrap();

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Flashlight Pipeline Layout"),
        bind_group_layouts: &[
            projection_matrix_bind_group_layout,
            FLASHLIGHT_BIND_GROUP_LAYOUT.get().unwrap(),
        ],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Flashlight Pipeline"),
        layout: Some(&pipeline_layout),
        cache: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("flashlight_vs_main"),
            buffers: &[ FlashlightVertex::desc() ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("flashlight_fs_main"),
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
    })
}

