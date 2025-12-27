use crate::{atlas::WgpuAtlas, prelude::*};
use crate::wgpu_engine::WgpuEngine;

pub(crate) fn create_standard_pipelines(
    device: &wgpu::Device,
    projection_matrix: &crate::ProjectionMatrix,
    atlas: &WgpuAtlas,
) -> Vec<(tataku::BlendMode, wgpu::RenderPipeline)> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Standard Shader"),
        source: wgpu::ShaderSource::Wgsl(crate::shader_files::SHADER.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("Standard Render Pipeline Layout"),
            bind_group_layouts: &[
                &projection_matrix.layout,
                &atlas.layout,
            ],
            push_constant_ranges: &[],
        }
    );


    let mut pipelines = Vec::new();
    for blend_mode in tataku::BlendMode::ALL.iter().copied() {
        let blend_state = WgpuEngine::map_blend_mode(blend_mode);

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


        pipelines.push((
            blend_mode, 
            pipeline
        ));
    }

    pipelines
}
