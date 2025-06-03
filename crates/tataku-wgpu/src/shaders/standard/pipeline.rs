use crate::prelude::*;
use tataku_engine::prelude::*;

pub fn create_standard_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    projection_matrix_bind_group_layout: &BindGroupLayout,
    texture_bind_group_layout: &BindGroupLayout,
) -> HashMap<Pipeline, RenderPipeline> {
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Standard Shader"),
        #[cfg(feature="texture_arrays")] 
        source: ShaderSource::Wgsl(crate::shader_files::SHADER_TEX_ARRAY.into()),
        #[cfg(not(feature="texture_arrays"))] 
        source: ShaderSource::Wgsl(crate::shader_files::SHADER.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(
        &PipelineLayoutDescriptor {
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
        Pipeline::AlphaBlending,
        Pipeline::AlphaOverwrite,
        Pipeline::PremultipliedAlpha,
        Pipeline::AdditiveBlending,
        Pipeline::OsuAdditiveBlending,
        Pipeline::SourceAlphaBlending,
    ] {
        let blend_state = WgpuEngine::map_blend_mode(blend_mode);

        let pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some(&format!("{blend_mode:?} Pipeline")),
                layout: Some(&render_pipeline_layout),
                cache: None,
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[ StandardVertex::desc() ],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(ColorTargetState {
                        format: config.format,
                        blend: Some(blend_state),
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
            }
        );

        pipelines.insert(blend_mode, pipeline);
    }

    pipelines
}
