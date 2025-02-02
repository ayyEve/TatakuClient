use crate::prelude::*;
use tataku_engine::prelude::*;
use wgpu::PipelineCompilationOptions;

pub fn create_standard_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> HashMap<BlendMode, wgpu::RenderPipeline> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Standard Shader"),
        #[cfg(feature="texture_arrays")] source: wgpu::ShaderSource::Wgsl(crate::shader_files::SHADER_TEX_ARRAY.into()),
        #[cfg(not(feature="texture_arrays"))] source: wgpu::ShaderSource::Wgsl(crate::shader_files::SHADER.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Standard Render Pipeline Layout"),
        bind_group_layouts: &[
            projection_matrix_bind_group_layout,
            texture_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });


    let mut pipelines = HashMap::new();
    for blend_mode in [
        BlendMode::AlphaBlending,
        BlendMode::AlphaOverwrite,
        BlendMode::PremultipliedAlpha,
        BlendMode::AdditiveBlending,
        BlendMode::OsuAdditiveBlending,
        BlendMode::SourceAlphaBlending,
    ] {
        let blend_state = WgpuEngine::map_blend_mode(blend_mode);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{blend_mode:?} Pipeline")),
            layout: Some(&render_pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[ StandardVertex::desc() ],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
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

        pipelines.insert(blend_mode, pipeline);
    }

    pipelines
}
