use tataku_client_common::prelude::*;
use crate::prelude::*;

pub fn create_flashlight_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    projection_matrix_bind_group_layout: &BindGroupLayout,
) -> RenderPipeline {
    let shader = device.create_shader_module(
        ShaderModuleDescriptor {
            label: Some("Flashlight Shader"),
            source: ShaderSource::Wgsl(crate::shader_files::FLASHLIGHT.into()),
        }
    );

    let bind_group_layout = device.create_bind_group_layout(
        &BindGroupLayoutDescriptor {
            label: Some("flashlight group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            size_of::<FlashlightDataInner>() as u64 * 2
                        )
                    },
                    count: None,
                },
            ],
        }
    );

    let pipeline_layout = device.create_pipeline_layout(
        &PipelineLayoutDescriptor {
            label: Some("Flashlight Pipeline Layout"),
            bind_group_layouts: &[
                projection_matrix_bind_group_layout,
                &bind_group_layout,
            ],
            push_constant_ranges: &[],
        }
    );

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Flashlight Pipeline"),
        layout: Some(&pipeline_layout),
        cache: None,
        vertex: VertexState {
            module: &shader,
            entry_point: Some("flashlight_vs_main"),
            buffers: &[ FlashlightVertex::desc() ],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("flashlight_fs_main"),
            targets: &[Some(ColorTargetState {
                format: config.format,
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
    })
}
