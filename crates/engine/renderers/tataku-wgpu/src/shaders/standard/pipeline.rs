use crate::prelude::*;
use std::collections::HashMap;
use crate::wgpu_engine::WgpuEngine;

pub(crate) fn create_standard_pipeline(
    device: &wgpu::Device,
    projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> HashMap<tataku::GraphicsPipeline, wgpu::RenderPipeline> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Standard Shader"),
        #[cfg(feature="texture_arrays")]
        source: wgpu::ShaderSource::Wgsl(crate::shader_files::SHADER_TEX_ARRAY.into()),
        #[cfg(not(feature="texture_arrays"))]
        source: wgpu::ShaderSource::Wgsl(crate::shader_files::SHADER.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Standard Render Pipeline Layout"),
            bind_group_layouts: &[
                projection_matrix_bind_group_layout,
                texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        }
    );


    let mut pipelines = HashMap::new();
    for blend_mode in [
        tataku::BlendMode::AlphaBlending,
        tataku::BlendMode::AlphaOverwrite,
        tataku::BlendMode::PremultipliedAlpha,
        tataku::BlendMode::AdditiveBlending,
        tataku::BlendMode::OsuAdditiveBlending,
        tataku::BlendMode::SourceAlphaBlending,
    ] {
        let blend_state = map_blend_mode(blend_mode);

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{blend_mode:?} Pipeline")),
                layout: Some(&render_pipeline_layout),
                cache: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[ super::Vertex::layout() ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: crate::FORMAT.remove_srgb_suffix(),
                        blend: Some(blend_state),
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
            }
        );


        pipelines.insert(
            tataku::GraphicsPipeline::Standard(blend_mode), 
            pipeline
        );
    }

    pipelines
}

pub(crate) fn map_blend_mode(blend_mode: tataku::BlendMode) -> wgpu::BlendState {
    use wgpu:: {
        BlendState,
        BlendComponent,
        BlendFactor,
        BlendOperation
    };

    match blend_mode {
        tataku::BlendMode::AlphaBlending => BlendState::ALPHA_BLENDING,
        tataku::BlendMode::AlphaOverwrite => BlendState::REPLACE,
        tataku::BlendMode::PremultipliedAlpha => BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        tataku::BlendMode::AdditiveBlending => BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            }
        },
        tataku::BlendMode::OsuAdditiveBlending => BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            }
        },
        tataku::BlendMode::SourceAlphaBlending => BlendState {
            color: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add
            }
        },
    }
}
