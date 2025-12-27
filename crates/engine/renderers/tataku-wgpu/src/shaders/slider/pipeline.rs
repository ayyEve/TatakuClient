use crate::prelude::*;
use crate::shaders::slider;

pub(crate) fn create_slider_pipeline(
    device: &wgpu::Device,
    projection_matrix: &ProjectionMatrix,
) -> wgpu::RenderPipeline {
    let slider_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Slider Shader"),
        source: wgpu::ShaderSource::Wgsl(crate::shader_files::SLIDER.into()),
    });

    let slider_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("slider group layout"),
            entries: &[
                // slider_data
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            size_of::<tataku::SliderData>() as u64 * 2
                        )
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
                        min_binding_size: NonZeroU64::new(size_of::<tataku::GridCell>() as u64 * 2)
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
                        min_binding_size: NonZeroU64::new(size_of::<u32>() as u64 * 2)
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
                        min_binding_size: NonZeroU64::new(
                            size_of::<tataku::LineSegment>() as u64 * 2
                        )
                    },
                    count: None,
                },

            ],
        }
    );

    let slider_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Slider Pipeline Layout"),
            bind_group_layouts: &[
                &projection_matrix.layout,
                &slider_bind_group_layout,
            ],
            push_constant_ranges: &[],
        }
    );

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Slider Pipeline"),
        layout: Some(&slider_pipeline_layout),
        cache: None,
        vertex: wgpu::VertexState {
            module: &slider_shader,
            entry_point: Some("slider_vs_main"),
            buffers: &[ slider::Vertex::layout() ],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &slider_shader,
            entry_point: Some("slider_fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: crate::FORMAT.remove_srgb_suffix(),
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
