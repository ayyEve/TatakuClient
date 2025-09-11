// WARNING: there is a lot of just data and setup code in this
use std::sync::Arc;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::prelude::*;
use crate::texture::WgpuTexture;
use crate::renderable_surface::*;

use tataku::Take as _;
use tataku::MatrixHelpers as _; 
use tataku::Interpolation as _;
use wgpu::util::DeviceExt as _;
use graphics::RenderingEngine as _;
use lyon_tessellation::geom::{ Box2D, Point };
use winit::raw_window_handle::{ HasWindowHandle, HasDisplayHandle };
use lyon_tessellation::path::{ builder::BorderRadii, Path as LyonPath };


// must not go past 16
const LAYER_COUNT:u32 = 12;
const MAX_DEPTH:f32 = 8192.0 * 8192.0;

/// background color
const GFX_CLEAR_COLOR:tataku::Color = tataku::Color::BLACK;

macro_rules! get_render_buffer {
    ($self: ident, $t: ident) => {{
        let b = $self.current_render_buffer
            .as_mut()
            .expect("last drawn type not set");

        if let RenderBufferQueueType::$t(b2) = &mut **b {b2}
            else { panic!("wrong buffer type") }
    }}
}

pub struct WgpuEngine<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,

    pipelines: HashMap<tataku::GraphicsPipeline, wgpu::RenderPipeline>,

    buffer_queues: HashMap<PipelineType, Box<RenderBufferQueueType>>,
    completed_buffers: Vec<RenderBufferType>,
    current_render_buffer: Option<Box<RenderBufferQueueType>>,

    projection_matrix: tataku::Matrix,
    projection_matrix_buffer: wgpu::Buffer,
    projection_matrix_bind_group: wgpu::BindGroup,

    atlas: tataku::Atlas,
    atlas_texture: WgpuTexture,

    screenshot_pending: Option<graphics::ScreenshotCallback>,

    // sampler: wgpu::Sampler,
    particle_system: shaders::particles::ParticleSystem,
    gaussian_blur_pipeline: shaders::gaussian_blur::Pipeline,
    box_blur_pipeline: shaders::box_blur::Pipeline,

    #[cfg(feature="vello_rendering")]
    vello_pipeline: Option<shaders::vello::Pipeline>,
    font_scale_context: parley::swash::scale::ScaleContext,


    pub(crate) scissors: tataku::ScissorManager,

    present_modes: Vec<tataku::Vsync>,
    can_blur: bool,
    blur_enabled: bool,

    intermediate_texture: Option<wgpu::Texture>,
    deferred_free_textures: Vec<tataku::TextureReference>,
}
impl<'window> WgpuEngine<'window> {

    // Creating some of the wgpu types requires async code
    pub async fn create<W:HasWindowHandle + HasDisplayHandle + Sync>(
        window: &'window W,
        settings: &engine::settings::display::DisplaySettings,
    ) -> Box<dyn graphics::RenderingEngine + 'window> {
        let window_size = settings.window_size;

        // create a wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL, // | Backends::GL,
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: None,
                for_device_loss: None
            },
            backend_options: wgpu::BackendOptions {
                gl: wgpu::GlBackendOptions {
                    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                    fence_behavior: wgpu::GlFenceBehavior::Normal,
                },
                dx12: wgpu::Dx12BackendOptions {
                    shader_compiler: wgpu::Dx12Compiler::default()
                },
                noop: wgpu::NoopBackendOptions { enable: false }
            },
        });

        // create the surface
        let surface: wgpu::Surface<'window> = instance
            .create_surface(window)
            .unwrap();

        // create the adapter
        use engine::settings::display::PerformanceMode;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: match settings.performance_mode {
                PerformanceMode::HighPerformance => wgpu::PowerPreference::HighPerformance,
                PerformanceMode::PowerSaver => wgpu::PowerPreference::LowPower,
            },
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        // create device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                #[cfg(feature="texture_arrays")]
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::BGRA8UNORM_STORAGE,
                #[cfg(not(feature="texture_arrays"))]
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits {
                    max_binding_array_elements_per_shader_stage: LAYER_COUNT,
                    ..Default::default()
                },
                memory_hints: wgpu::MemoryHints::Performance,
                label: Some("device request"),
                trace: wgpu::Trace::Off
            },
        ).await.unwrap();

        let can_blur = device.features().contains(wgpu::Features::BGRA8UNORM_STORAGE);
        if !can_blur { warn!("Blur unsupported on this device!"); }

        // no more comments good luck!
        let surface_caps = surface.get_capabilities(&adapter);
        let present_modes = surface_caps
            .present_modes
            .into_iter()
            .map(VsyncUtils::map_to_vsync)
            .chain([tataku::Vsync::AutoNoVsync, tataku::Vsync::AutoVsync])
            .collect();


        let formats = surface_caps.formats.clone();

        let mut surface_format = crate::FORMAT;
        if !formats.contains(&surface_format) {
            error!("no rgba8unorm srgb!!!");

            surface_format = formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(formats[0]);
        }


        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size[0] as u32,
            height: window_size[1] as u32,
            present_mode: wgpu::PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],

            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        #[cfg(feature="texture_arrays")]
        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("atlas group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: NonZeroU32::new(LAYER_COUNT),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }
        );

        #[cfg(not(feature="texture_arrays"))]
        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("atlas group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            }
        );

        let proj_matrix_size = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
        let projection_matrix_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture/Sampler bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(proj_matrix_size)
                        },
                        count: None,
                    },
                ]
            }
        );

        let window_size = tataku::Vector2::new(window_size[0], window_size[1]);
        let projection_matrix = Self::create_projection(window_size);
        let projection_matrix_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Projection Matrix Buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&projection_matrix.to_raw()),
            }
        );

        let projection_matrix_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("diffuse_bind_group"),
                layout: &projection_matrix_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &projection_matrix_buffer,
                            offset: 0,
                            size: NonZeroU64::new(proj_matrix_size)
                        }),
                    },
                ],
            }
        );

        // because the swapchain texture can only have RenderAttachment (**annoy**)
        // we render to an intermediary texture, which can have blur applied and used for screenshots
        let intermediate_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("Render Texture"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: crate::FORMAT.remove_srgb_suffix(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[ 
                    crate::FORMAT.add_srgb_suffix(),
                    crate::FORMAT.remove_srgb_suffix() 
                ]
            }
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let pipelines = Self::init_pipelines(
            &device, 
            &texture_bind_group_layout, 
            &projection_matrix_bind_group_layout,
        );
        

        let intermediate_tex_ref = WgpuTextureReference::new(&intermediate_texture);
        #[cfg(feature="vello_rendering")]
        let vello_pipeline = shaders::vello::Pipeline::create(&device, &intermediate_tex_ref);

        let gaussian_blur_pipeline = shaders::gaussian_blur::Pipeline::new(&device, &intermediate_tex_ref);
        let box_blur_pipeline = shaders::box_blur::Pipeline::new(&device, &intermediate_tex_ref);
        
        let particle_system = shaders::particles::ParticleSystem::new(&device);

        let atlas_size = device.limits().max_texture_dimension_2d.min(8192);
        let atlas_texture = Self::create_texture(
            &device,
            &texture_bind_group_layout,
            &sampler,
            atlas_size,
            atlas_size,
            surface_format,
        );

        let buffer_queues = Self::init_buffer_queues(
            &device, 
            &pipelines,
            &gaussian_blur_pipeline.pipeline,
            &box_blur_pipeline.pipeline,
        );

        Box::new(Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            pipelines,
            atlas: tataku::Atlas::new(
                atlas_size,
                atlas_size,
                LAYER_COUNT
            ),
            atlas_texture,

            current_render_buffer: None,
            buffer_queues,
            completed_buffers: Vec::new(),

            projection_matrix,
            projection_matrix_buffer,
            projection_matrix_bind_group,
            screenshot_pending: None,

            particle_system,
            gaussian_blur_pipeline,
            box_blur_pipeline,

            #[cfg(feature="vello_rendering")]
            vello_pipeline,
            font_scale_context: parley::swash::scale::ScaleContext::new(),

            scissors: tataku::ScissorManager::default(),
            present_modes,
            // sampler,
            can_blur,
            blur_enabled: true,
            intermediate_texture: Some(intermediate_texture),

            deferred_free_textures: Vec::new(),
        })
    }

    fn init_pipelines(
        device: &wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        projection_matrix_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> HashMap<tataku::GraphicsPipeline, wgpu::RenderPipeline> {
        let mut pipelines = shaders::standard::create_standard_pipeline(
            device,
            projection_matrix_bind_group_layout,
            texture_bind_group_layout
        );

        // create slider pipeline
        pipelines.insert(tataku::GraphicsPipeline::Slider, shaders::slider::create_slider_pipeline(
            device,
            projection_matrix_bind_group_layout
        ));

        // create flashlight pipeline
        pipelines.insert(tataku::GraphicsPipeline::Flashlight, shaders::flashlight::create_flashlight_pipeline(
            device,
            projection_matrix_bind_group_layout
        ));

        pipelines
    }
    
    fn init_buffer_queues(
        device: &wgpu::Device,
        pipelines: &HashMap<tataku::GraphicsPipeline, wgpu::RenderPipeline>,
        gaussian_blur_pipeline: &wgpu::ComputePipeline,
        box_blur_pipeline: &wgpu::ComputePipeline,
    ) -> HashMap<PipelineType, Box<RenderBufferQueueType>> {
        [
            (PipelineType::Standard, Box::new(RenderBufferQueueType::Standard(
                RenderBufferQueue::default().init(
                    device,
                    &pipelines[&tataku::GraphicsPipeline::Standard(tataku::BlendMode::AlphaBlending)]
                )
            ))),
            (PipelineType::Slider, Box::new(RenderBufferQueueType::Slider(
                RenderBufferQueue::default().init(
                    device,
                    &pipelines[&tataku::GraphicsPipeline::Slider]
                )
            ))),
            (PipelineType::Flashlight, Box::new(RenderBufferQueueType::Flashlight(
                RenderBufferQueue::default().init(
                    device,
                    &pipelines[&tataku::GraphicsPipeline::Flashlight]
                )
            ))),

            
            (PipelineType::GaussianBlur, Box::new(RenderBufferQueueType::GaussianBlur(
                RenderBufferQueue::default().init(
                    device,
                    gaussian_blur_pipeline
                )
            ))),
            (PipelineType::BoxBlur, Box::new(RenderBufferQueueType::BoxBlur(
                RenderBufferQueue::default().init(
                    device,
                    box_blur_pipeline
                )
            ))),

            #[cfg(feature="vello")]
            (PipelineType::Vello, Box::new(RenderBufferQueueType::Vello(
                RenderBufferQueue::default().init(
                    device,
                    WgpuPipeline::None
                )
            ))),
        ].into_iter().collect()
    }


    pub fn render_current_surface(&mut self) -> Result<(), wgpu::SurfaceError> {
        let swapchain = self.surface.get_current_texture()?;
        let size = swapchain.texture.size();

        let mut texture = self.intermediate_texture.take().unwrap();

        // don't draw if our draw surface has no area
        if size.width == 0 || size.height == 0 { return Ok(()) }

        if texture.size() != size {
            let format = texture.format().remove_srgb_suffix();
            texture = self.device.create_texture(
                &wgpu::TextureDescriptor {
                    label: Some("Intermediate Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: texture.dimension(),
                    format,
                    usage: texture.usage(),
                    view_formats: &[ format.add_srgb_suffix() ]
                }
            );
        }


        let tex = WgpuTextureReference::new(
            &texture
        );
        self.render(&RenderableSurface::new(
            &tex,
            GFX_CLEAR_COLOR,
            tataku::Vector2::new(size.width as f32, size.height as f32),
        ))?;

        // `texture` should now have our data, with which we can use to render the surface, as well as use for screenshots
        // again though, because the swapchain texture can only be rendered to directly for some reason, we have to use a shader
        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let view = wgpu::TextureViewDescriptor::default();
        wgpu::util::TextureBlitter::new(
            &self.device,
            swapchain.texture.format(),
        ).copy(
            &self.device,
            &mut encoder,
            &texture.create_view(&view),
            &swapchain.texture.create_view(&view)
        );
        
        let i = self.queue.submit([encoder.finish()]);
        self.device.poll(wgpu::wgt::PollType::WaitForSubmissionIndex(i)).unwrap();
        swapchain.present();

        if let Some(screenshot) = self.screenshot_pending.take() {
            let (data, size) = self.read_texture(
                &texture
            );

            screenshot((data, size));
        }

        for i in self.deferred_free_textures.take() {
            self.free_tex(i, false);
        }

        self.intermediate_texture = Some(texture);

        Ok(())
    }

    fn render(&mut self, renderable: &RenderableSurface) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &renderable.texture.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(renderable.get_clear_color()),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                }
            );

            let mut current_pipeline = tataku::GraphicsPipeline::None;
            let mut current_scissor = None;

            for i in self.completed_buffers.iter() {
                let pipeline_type = i.get_pipeline_type();

                // blurs are a special case, they're compute shaders and not fragment shaders
                // vello is also a special case as it handles its own pipelines itself
                if pipeline_type.is_compute() {
                    if pipeline_type.is_blur()
                    && (!self.can_blur || !self.blur_enabled) {
                        continue
                    }

                    #[cfg(feature="vello")]
                    if pipeline_type.is_vello() && self.vello_pipeline.is_none() {
                        continue
                    }

                    // finish and submit the current render pass to free up the encoder
                    drop(render_pass);
                    self.queue.submit([encoder.finish()]);


                    // perform the compute shader
                    match pipeline_type {
                        PipelineType::GaussianBlur => {
                            let RenderBufferType::GaussianBlur(buffer) = i
                            else { unreachable!() };

                            self.gaussian_blur_pipeline.perform(
                                &self.device,
                                &self.queue,
                                renderable.texture,
                                buffer
                            );
                        }
                        PipelineType::BoxBlur => {
                            let RenderBufferType::BoxBlur(buffer) = i
                            else { unreachable!() };

                            self.box_blur_pipeline.perform(
                                &self.device,
                                &self.queue,
                                renderable.texture,
                                buffer
                            );
                        }

                        #[cfg(feature="vello")]
                        PipelineType::Vello => {
                            let vello = self
                                .vello_pipeline
                                .as_mut()
                                .unwrap();

                            let RenderBufferType::Vello(buffer) = i
                            else { unreachable!() };

                            vello.perform(
                                &self.device,
                                &self.queue,
                                renderable.texture,
                                buffer
                            );
                        }

                        _ => unreachable!()
                    }


                    // back to our regularly scheduled programming
                    encoder = self.device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder")
                        }
                    );

                    render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &renderable.texture.view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store, // must be store for render targets to work apparently
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    current_pipeline = tataku::GraphicsPipeline::None;
                    continue
                }

                let scissor = i.get_scissor();
                if scissor != current_scissor {
                    current_scissor = scissor;
                    let [x, y, w, h] = current_scissor
                        .unwrap_or_else(|| [0.0, 0.0, renderable.size.x, renderable.size.y]);

                    if renderable.size.x - x < 0.0 || renderable.size.y - y < 0.0 {
                        continue
                    }

                    let x = x.clamp(0.0, renderable.size.x);
                    let y = y.clamp(0.0, renderable.size.y);

                    render_pass.set_scissor_rect(
                        x as u32,
                        y as u32,
                        w.clamp(0.0, renderable.size.x - x) as u32,
                        h.clamp(0.0, renderable.size.y - y) as u32
                    );
                }

                let pipeline = i.get_pipeline();

                if pipeline != current_pipeline {
                    current_pipeline = pipeline;
                    let Some(pipeline) = self.pipelines.get(&pipeline)
                    else {
                        error!("Pipeline not created for blend mode {current_pipeline:?}");
                        current_pipeline = tataku::GraphicsPipeline::None;
                        continue
                    };

                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(
                        0,
                        &self.projection_matrix_bind_group,
                        &[]
                    );

                    if let RenderBufferType::Standard(_) = i {
                        render_pass.set_bind_group(
                            1,
                            &self.atlas_texture.bind_group,
                            &[]
                        );
                    }
                }

                if let RenderBufferType::Slider(slider) = i {
                    render_pass.set_bind_group(
                        1,
                        &slider.bind_group,
                        &[]
                    );
                }
                if let RenderBufferType::Flashlight(flashlight) = i {
                    render_pass.set_bind_group(
                        1,
                        &flashlight.bind_group,
                        &[]
                    );
                }

                render_pass.set_vertex_buffer(
                    0,
                    i.get_vertex_buffer().slice(..)
                );
                render_pass.set_index_buffer(
                    i.get_index_buffer().slice(..),
                    wgpu::IndexFormat::Uint32
                );
                render_pass.draw_indexed(
                    0..i.get_used_indices() as u32,
                    0,
                    0..1
                );
            }

        }

        self.queue.submit([encoder.finish()]);

        Ok(())
    }

    fn create_projection(draw_size: tataku::Vector2) -> tataku::Matrix {
        let sx = 2.0 / draw_size.x;
        let sy = -2.0 / draw_size.y;

        // setup depth range
        let far = MAX_DEPTH;
        let near = -far;
        let depth_range = 1.0 / (far - near);

        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, depth_range, 0.0],
            [-1.0, 1.0, -near * depth_range, 1.0]
        ].into()
    }

}

// texture stuff
impl WgpuEngine<'_> {
    fn create_texture(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> WgpuTexture {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureViewDescriptor {
            label: Some("atlas_texture_view"),
            ..Default::default()
        };

        let view_formats = [
            format.add_srgb_suffix(), 
            format.remove_srgb_suffix() 
        ];

        let textures = (0..LAYER_COUNT).map(|_| {
            let texture = device.create_texture(
                &wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    // Most images are stored using sRGB so we need to reflect that here.
                    format, //TextureFormat::Rgba8UnormSrgb,
                    // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    label: Some("atlas_texture"),
                    view_formats: &view_formats,
                }
            );

            let view = texture.create_view(&desc);
            (texture, view)
        })
        .collect::<Vec<_>>();


        #[cfg(feature="texture_arrays")]
        let view_list = textures.iter()
            .map(|a| &a.1)
            .collect::<Vec<_>>();
        
        #[cfg(feature="texture_arrays")]
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("texture array bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureViewArray(&view_list),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    }
                ],
            }
        );

        #[cfg(not(feature="texture_arrays"))]
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture array bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&textures[0].1),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&textures[1].1),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&textures[2].1),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&textures[3].1),
                },
            ],
        });

        WgpuTexture {
            textures: Arc::new(textures),
            bind_group
        }
    }

    fn read_texture(&self, texture: &wgpu::Texture) -> (Vec<u8>, [u32;2]) {
        let (w, h) = (texture.width(), texture.height());

        let fuck = (w * 4)
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

        let size = (fuck * h) as u64; //(w * h * 4) as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Reading Buffer"),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            size,
            mapped_at_creation: false,
        });

        let tex_buffer = wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(fuck),
                rows_per_image: None
            }
        };

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Texture reading encoder") }
        );
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            tex_buffer,
            texture.size()
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        let index = self.queue.submit(None);
        self.device.poll(wgpu::wgt::PollType::WaitForSubmissionIndex(index)).unwrap();

        let data = slice
            .get_mapped_range()
            .chunks_exact(4)
            .flat_map(|b| cast_to_rgba_bytes(b, texture.format()))
            .collect();

        (data, [fuck / 4, h])
    }

}


// render code
impl WgpuEngine<'_> {
    fn dump_last_drawn(&mut self) {
        let Some(mut last_buffer) = self
            .current_render_buffer.take()
        else { return };

        let pipeline = match last_buffer.pipeline_type() {
            PipelineType::GaussianBlur => WgpuPipeline::Compute(&self.gaussian_blur_pipeline.pipeline),
            PipelineType::BoxBlur => WgpuPipeline::Compute(&self.box_blur_pipeline.pipeline),
            PipelineType::Vello => WgpuPipeline::None,
            _ => WgpuPipeline::Render(&self.pipelines[&last_buffer.graphics_pipeline()]),
        };
        if let Some(b) = last_buffer.dump_and_next(
            &self.queue,
            &self.device,
            pipeline
        ) {
            self.completed_buffers.push(b);
        };

        self.buffer_queues.insert(last_buffer.pipeline_type(), last_buffer);
    }

    fn check_dump_and_next(&mut self, to_draw: PipelineType) {
        if let Some(last_buffer) = &self.current_render_buffer
            && last_buffer.pipeline_type() == to_draw
        { return }

        self.dump_last_drawn();
        self.current_render_buffer = Some(self.buffer_queues
            .remove(&to_draw)
            .unwrap_or_else(|| panic!("buffer queue did not have a queue for type {to_draw:?}. Did you forget to create a buffer queue for it?"))
        );
    }

    /// returns reserve data
    fn reserve_standard<'a>(
        &'a mut self,
        vtx_count: u64,
        idx_count: u64,
        blend_mode: tataku::BlendMode
    ) -> Option<shaders::standard::ReserveData<'a>> {
        use crate::shaders::standard;

        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Standard);

        let vertex_buffer_queue = get_render_buffer!(self, Standard);

        let mut recording_buffer = vertex_buffer_queue
            .recording_buffer()
            .expect("didnt get vertex recording buffer");

        if !( // blend mode check
            recording_buffer.blend_mode.is_none()
            || recording_buffer.blend_mode.unwrap() == blend_mode
        )
        || !( // scissor check
            recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none()
        )
        || recording_buffer.used_vertices + vtx_count > standard::Buffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > standard::Buffer::IDX_PER_BUF {
            let pipeline = WgpuPipeline::Render(
                &self.pipelines[&tataku::GraphicsPipeline::Standard(blend_mode)]
            );

            if let Some(b) = vertex_buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                pipeline
            ) {
                self.completed_buffers.push(RenderBufferType::Standard(b));
            }

            recording_buffer = vertex_buffer_queue.recording_buffer()?;
            recording_buffer.blend_mode = Some(blend_mode);
            recording_buffer.scissor = Some(scissor);
        }
        if recording_buffer.blend_mode.is_none() {
            recording_buffer.blend_mode = Some(blend_mode);
        }
        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used_indices += idx_count;
        recording_buffer.used_vertices += vtx_count;

        let used_vertices = recording_buffer.used_vertices;
        let used_indices = recording_buffer.used_indices;

        let cache = &mut vertex_buffer_queue.cpu_cache;
        Some(standard::ReserveData {
            vtx: &mut cache.cpu_vtx[
                (used_vertices - vtx_count) as usize .. used_vertices as usize
            ],
            idx: &mut cache.cpu_idx[
                (used_indices - idx_count) as usize .. used_indices as usize
            ],
            idx_offset: used_vertices - vtx_count,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn reserve_tex_quad(
        &mut self,
        tex: &tataku::TextureReference,
        rect: [f32; 4],
        color: tataku::Color,
        h_flip: bool,
        v_flip: bool,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
    ) {
        use shaders::standard;
        let Some(mut reserved) = self.reserve_standard(
            4,
            6,
            blend_mode
        ) else { return };

        let [x, y, w, h] = rect;
        let color = color.into();

        let mut tl = tex.uvs.tl;
        let mut tr = tex.uvs.tr;
        let mut bl = tex.uvs.bl;
        let mut br = tex.uvs.br;

        if h_flip {
            std::mem::swap(&mut tl, &mut tr);
            std::mem::swap(&mut bl, &mut br);
        }
        if v_flip {
            std::mem::swap(&mut tl, &mut bl);
            std::mem::swap(&mut tr, &mut br);
        }

        let tex_index = tex.layer as i32;
        let offset = reserved.idx_offset as u32;
        #[allow(clippy::identity_op, reason = "lines the values up nicely")]
        reserved.copy_in(
        &[
                standard::Vertex {
                    position: transform.mul_v2(tataku::Vector2::new(x, y)).into(),
                    tex_coords: tl,
                    tex_index,
                    color,
                },
                standard::Vertex {
                    position: transform.mul_v2(tataku::Vector2::new(x+w, y)).into(),
                    tex_coords: tr,
                    tex_index,
                    color,
                },
                standard::Vertex {
                    position: transform.mul_v2(tataku::Vector2::new(x, y+h)).into(),
                    tex_coords: bl,
                    tex_index,
                    color,
                },
                standard::Vertex {
                    position: transform.mul_v2(tataku::Vector2::new(x+w, y+h)).into(),
                    tex_coords: br,
                    tex_index,
                    color,
                }
            ],
            &[
                0 + offset,
                2 + offset,
                1 + offset,

                1 + offset,
                2 + offset,
                3 + offset,
            ]
        );
    }

    // quad is tl,tr, bl,br
    fn reserve_quad(
        &mut self,
        quad: [tataku::Vector2; 4],
        color: tataku::Color,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
    ) {
        let Some(mut reserved) = self.reserve_standard(
            4,
            6,
            blend_mode
        ) else { return };
        let color = color.into();

        let vertices = quad.into_iter()
            .map(|p| shaders::standard::Vertex {
                position: transform.mul_v2(p).into(),
                color,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let offset = reserved.idx_offset as u32;
        #[allow(clippy::identity_op, reason = "lines the values up nicely")]
        reserved.copy_in(&vertices, &[
            0 + offset,
            2 + offset,
            1 + offset,

            1 + offset,
            2 + offset,
            3 + offset,
        ]);
    }

    pub(crate) fn reserve_slider<'a>(
        &'a mut self,
        slider_grid_count: u64,
        grid_cell_count: u64,
        line_segment_count: u64,
    ) -> Option<shaders::slider::ReserveData<'a>> {
        use shaders::slider;

        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Slider);

        let slider_buffer_queue = get_render_buffer!(self, Slider);

        let mut recording_buffer = slider_buffer_queue
            .recording_buffer()
            .expect("didnt get slider recording buffer");

        let scissor_check = recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none();


        let vtx_count = 4;
        let idx_count = 6;

        // FIXME:
        // assert!(slider_grid_count < SLIDER_GRID_COUNT);
        // assert!(grid_cell_count < GRID_CELL_COUNT);
        // assert!(line_segment_count < LINE_SEGMENT_COUNT);

        if slider_grid_count > slider::SLIDER_GRID_COUNT
        || grid_cell_count > slider::GRID_CELL_COUNT
        || line_segment_count > slider::LINE_SEGMENT_COUNT {
            return None
        }

        if !scissor_check
        || recording_buffer.used_vertices + vtx_count > slider::Buffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > slider::Buffer::IDX_PER_BUF
        || recording_buffer.used_slider_data + 1 > slider::EXPECTED_SLIDER_COUNT
        || recording_buffer.used_slider_grids + slider_grid_count > slider::SLIDER_GRID_COUNT
        || recording_buffer.used_grid_cells + grid_cell_count > slider::GRID_CELL_COUNT
        || recording_buffer.used_line_segments + line_segment_count > slider::LINE_SEGMENT_COUNT
        {
            let pipeline = WgpuPipeline::Render(
                &self.pipelines[&tataku::GraphicsPipeline::Slider]
            );
            if let Some(b) = slider_buffer_queue
            .dump_and_next(
                &self.queue,
                &self.device,
                pipeline
            ) {
                self.completed_buffers.push(RenderBufferType::Slider(b));
            }
            recording_buffer = slider_buffer_queue.recording_buffer()?;
        }

        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used_indices += idx_count;
        recording_buffer.used_vertices += vtx_count;

        recording_buffer.used_slider_data += 1;
        recording_buffer.used_slider_grids += slider_grid_count;
        recording_buffer.used_grid_cells += grid_cell_count;
        recording_buffer.used_line_segments += line_segment_count;

        let used_vertices = recording_buffer.used_vertices as usize;
        let used_indices = recording_buffer.used_indices as usize;

        let used_slider_grids = recording_buffer.used_slider_grids as usize;
        let used_grid_cells = recording_buffer.used_grid_cells as usize;
        let used_line_segments = recording_buffer.used_line_segments as usize;

        // reserve slider vertex data
        let slider_index = recording_buffer.used_slider_data - 1;

        let cache = &mut slider_buffer_queue.cpu_cache;
        Some(slider::ReserveData {
            vtx: &mut cache.cpu_vtx[
                (used_vertices - vtx_count as usize) .. used_vertices
            ],
            idx: &mut cache.cpu_idx[
                (used_indices - idx_count as usize) .. used_indices
            ],

            slider_data: &mut cache.slider_data[slider_index as usize],
            slider_grids: &mut cache.slider_grids[
                (used_slider_grids - slider_grid_count as usize) .. used_slider_grids
            ],
            grid_cells: &mut cache.grid_cells[
                (used_grid_cells - grid_cell_count as usize) .. used_grid_cells
                ],
            line_segments: &mut cache.line_segments[
                (used_line_segments - line_segment_count as usize) .. used_line_segments
            ],

            idx_offset: used_vertices as u64 - vtx_count,
            slider_index: slider_index as u32,
            slider_grid_offset: used_slider_grids as u32 - slider_grid_count as u32,
            grid_cell_offset: used_grid_cells as u32 - grid_cell_count as u32,
            line_segment_offset: used_line_segments as u32 - line_segment_count as u32,
        })
    }

    pub(crate) fn reserve_flashlight<'a>(
        &'a mut self,
    ) -> Option<shaders::flashlight::ReserveData<'a>> {
        use shaders::flashlight;
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Flashlight);

        let buffer_queue = get_render_buffer!(self, Flashlight);
        // if let Some(RenderBufferQueueType::Slider(b)) = &mut self.last_drawn {b} else {panic!("wrong buffer type")};

        let mut recording_buffer = buffer_queue
            .recording_buffer()
            .expect("didnt get flashlight recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none();

        let vtx_count = 4;
        let idx_count = 6;

        if !scissor_check
        || recording_buffer.used_vertices + vtx_count > flashlight::Buffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > flashlight::Buffer::IDX_PER_BUF
        {
            let pipeline = WgpuPipeline::Render(
                &self.pipelines[&tataku::GraphicsPipeline::Flashlight]
            );
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                pipeline
            ) {
                self.completed_buffers.push(RenderBufferType::Flashlight(b));
            }
            recording_buffer = buffer_queue.recording_buffer()?;
        }
        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used_flashlights += 1;
        recording_buffer.used_vertices += vtx_count;
        recording_buffer.used_indices += idx_count;


        // reserve flashlight vertex data
        let flashlight_index = recording_buffer.used_flashlights - 1;
        let used_vertices = recording_buffer.used_vertices;
        let used_indices = recording_buffer.used_indices;

        let cache = &mut buffer_queue.cpu_cache;
        Some(flashlight::ReserveData {
            vtx: &mut cache.cpu_vtx[(used_vertices - vtx_count) as usize .. used_vertices as usize],
            idx: &mut cache.cpu_idx[(used_indices - idx_count) as usize .. used_indices as usize],
            flashlight_data: &mut cache.cpu_flashlights[flashlight_index as usize],

            idx_offset: used_vertices - vtx_count,
            flashlight_index: flashlight_index as u32,
        })
    }

    pub(crate) fn reserve_gaussian_blur<'a>(
        &'a mut self,
    ) -> Option<shaders::gaussian_blur::ReserveData<'a>> {
        use shaders::gaussian_blur as gaussian;
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(PipelineType::GaussianBlur);

        let buffer_queue = get_render_buffer!(self, GaussianBlur);

        let mut recording_buffer = buffer_queue.recording_buffer()
            .expect("didnt get blur recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none();

        if !scissor_check
            || recording_buffer.used + 1 > gaussian::Buffer::VTX_PER_BUF
        {
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                (&self.gaussian_blur_pipeline.pipeline).into()
            ) {
                self.completed_buffers.push(RenderBufferType::GaussianBlur(b));
            }
            recording_buffer = buffer_queue.recording_buffer()?;
        }
        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used += 1;
        let index = recording_buffer.used - 1;

        let cache = &mut buffer_queue.cpu_cache;
        Some(gaussian::ReserveData {
            data: &mut cache.cpu_blurs[index as usize],
            _blur_index: index as u32,
        })
    }

    pub(crate) fn reserve_box_blur<'a>(
        &'a mut self,
    ) -> Option<shaders::box_blur::ReserveData<'a>> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(PipelineType::BoxBlur);

        let buffer_queue = get_render_buffer!(self, BoxBlur);

        let mut recording_buffer = buffer_queue.recording_buffer()
            .expect("didnt get blur recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none();

        if !scissor_check || recording_buffer.used {
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                (&self.box_blur_pipeline.pipeline).into(),
            ) {
                self.completed_buffers.push(RenderBufferType::BoxBlur(b));
            }
            recording_buffer = buffer_queue.recording_buffer()?;
        }
        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used = true;

        let cache = &mut buffer_queue.cpu_cache;
        Some(shaders::box_blur::ReserveData {
            data: &mut cache.cpu_blurs[0],
        })
    }

    #[cfg(feature="vello_rendering")]
    pub(crate) fn reserve_vello<'a>(
        &'a mut self,
    ) -> Option<shaders::vello::ReserveData<'a>> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Vello);

        let buffer_queue = get_render_buffer!(self, Vello);

        let mut recording_buffer = buffer_queue
            .recording_buffer()
            .expect("didnt get vello recording buffer");

        let scissor_check = 
            recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none()
            ;

        if !scissor_check {
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                WgpuPipeline::None
            ) {
                self.completed_buffers.push(RenderBufferType::Vello(b));
            }
            recording_buffer = buffer_queue.recording_buffer()?;
        }

        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used += 1;

        Some(shaders::vello::ReserveData {
            scene: &mut buffer_queue.cpu_cache.scene,
        })
    }

}


// draw helpers
impl WgpuEngine<'_> {
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

    fn tessellate_polygon(
        &mut self,
        polygon: &[tataku::Vector2],
        color: tataku::Color,
        border: Option<f32>,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode
    ) {
        let mut polygon = polygon.iter();
        let mut path = LyonPath::builder();
        path.begin(
            polygon.next().map(|p| Point::new(p.x, p.y)).unwrap()
        );

        for p in polygon {
            if !p.x.is_normal() || !p.y.is_normal() { return }
            path.line_to(Point::new(p.x, p.y));
        }
        path.end(true);
        let path = path.build();

        self.tessellate_path(&path, color, border, transform, blend_mode);
    }

    fn tessellate_path(
        &mut self,
        path: &LyonPath,
        color: tataku::Color,
        border: Option<f32>,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode
    ) {
        use lyon_tessellation::{
            VertexBuffers,
            StrokeOptions,
            FillTessellator,
            FillOptions,
            StrokeTessellator,
            geometry_builder::simple_builder,
        };

        // Create the destination vertex and index buffers.
        let mut buffers: VertexBuffers<Point<f32>, u16> = VertexBuffers::new();

        {
            let mut vertex_builder = simple_builder(&mut buffers);

            let result = if let Some(radius) = border {
                // Create the tessellator.
                let mut tessellator = StrokeTessellator::new();

                // Compute the tessellation.
                tessellator.tessellate_path(
                    path,
                    &StrokeOptions::default().with_line_width(radius * 2.0),
                    &mut vertex_builder
                )
            } else {
                // Create the tessellator.
                let mut tessellator = FillTessellator::new();

                // Compute the tessellation.
                tessellator.tessellate_path(
                    path,
                    &FillOptions::default(),
                    &mut vertex_builder
                )
            };
            if let Err(e) = result {
                error!("Tesselator error: {e}");
                return;
            }

        }

        let mut reserved = self.reserve_standard(
            buffers.vertices.len() as u64,
            buffers.indices.len() as u64,
            blend_mode
        ).expect("nope");

        // convert vertices and indices to their proper values
        let vertices = buffers.vertices
            .into_iter()
            .map(|n|
                shaders::standard::Vertex {
                    position: [n.x, n.y],
                    color: [color.r(), color.g(), color.b(), color.a()],
                    ..Default::default()
                }.apply_matrix(&transform)
            )
            .collect::<Vec<_>>();

        // insert the vertices and indices into the render buffer
        let indices = buffers.indices
            .into_iter()
            .map(|a| reserved.idx_offset as u32 + a as u32)
            .collect::<Vec<_>>();

        reserved.copy_in(&vertices, &indices);
    }
}


impl graphics::RenderingEngine for WgpuEngine<'_> {
    fn resize(
        &mut self,
        [width, height]: [u32; 2]
    ) {
        if width == 0 || height == 0 { return }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let window_size = tataku::Vector2::new(width as f32, height as f32);
        self.projection_matrix = Self::create_projection(window_size);
        self.queue.write_buffer(
            &self.projection_matrix_buffer,
            0,
            bytemuck::cast_slice(&self.projection_matrix.to_raw())
        );
    }

    fn set_vsync(&mut self, vsync: tataku::Vsync) {
        self.config.present_mode = VsyncUtils::map_from_vsync(
            vsync.to_okay(&self.present_modes)
        );
        self.surface.configure(&self.device, &self.config);
    }

    fn vsync_modes(&self) -> Vec<tataku::Vsync> {
        self.present_modes.clone()
    }

    fn set_blur(&mut self, enabled: bool) {
        self.blur_enabled = enabled;
    }


    fn dump_atlas(&self, path: &str) {
        std::fs::create_dir_all(path).unwrap();

        for (n, (tex, _)) in self.atlas_texture
            .textures
            .iter()
            .enumerate()
        {
            println!("Reading atlas {n}");
            let (data, [width, height]) = self.read_texture(tex);

            let path = format!("{path}/atlas_{n}.png");
            let file = std::fs::File::create(&path).unwrap();
            let png = image::codecs::png::PngEncoder::new(file);

            use image::ImageEncoder;
            png.write_image(
                &data,
                width,
                height,
                image::ExtendedColorType::Rgba8,
            ).unwrap();
        }

    }


    fn create_render_target(
        &mut self,
        [width, height]: [u32; 2],
        clear_color: tataku::Color,
        do_render: graphics::RenderTargetDraw,
    ) -> Option<graphics::RenderTarget> {
        // find space in the render target atlas
        let atlased = self.atlas.try_insert(width, height)?;

        // create a projection and render target
        let projection = Self::create_projection(
            tataku::Vector2::new(width as f32, height as f32)
        );

        let target = graphics::RenderTarget {
            width,
            height,
            projection,
            clear_color,
            image: graphics::Image::new(
                tataku::Vector2::ZERO,
                Arc::new(atlased),
                tataku::Vector2::ONE
            ),
        };

        // queue rendering the data to it
        self.update_render_target(target.clone(), do_render);

        // return the new render target
        Some(target)
    }
    fn update_render_target(
        &mut self,
        target: graphics::RenderTarget,
        do_render: graphics::RenderTargetDraw
    ) {
        if !tataku::Bounds::new(tataku::Vector2::ZERO, target.image.size()).has_area() {
            return
        }

        // get the texture this target was written to
        let textures = self.atlas_texture
            .textures
            .clone();

        let Some((atlas_tex, _)) = textures.get(
            target.image.tex.layer as usize
        )
        else { return };

        // write the projection matrix
        self.queue.write_buffer(
            &self.projection_matrix_buffer,
            0,
            bytemuck::cast_slice(&target.projection.to_raw())
        );
        self.queue.submit([]);

        let width = target.width;
        let height = target.height;

        // create a temporary texture to render this target to
        let texture = self.device.create_texture(
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: crate::FORMAT,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("render_target_temp_tex"),
                view_formats: &[ 
                    crate::FORMAT.add_srgb_suffix(),
                    crate::FORMAT.remove_srgb_suffix(),
                ],
            }
        );

        // create renderable surface
        let tex = WgpuTextureReference::new(&texture);
        let renderable = RenderableSurface::new(
            &tex,
            target.clear_color,
            tataku::Vector2::new(width as f32, height as f32),
        );

        // clear buffers
        self.begin_render();

        // fill buffers
        do_render(self, tataku::Matrix::identity());

        // finish up
        self.end_render();

        // perform render
        if let Err(e) = self.render(&renderable) {
            error!("Error rendering render target: {e:?}");
        }


        // copy render to atlas
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("render_target copy encoder"),
            }
        );

        let mut dest = atlas_tex.as_image_copy();
        dest.origin.x = target.image.tex.x;
        dest.origin.y = target.image.tex.y;

        encoder.copy_texture_to_texture(
            texture.as_image_copy(),
            dest,
            texture.size(),
        );
        self.queue.submit([encoder.finish()]);

        // remove temp texture
        self.queue.on_submitted_work_done(move || texture.destroy());

        // reapply the window projection matrix
        self.queue.write_buffer(
            &self.projection_matrix_buffer,
            0,
            bytemuck::cast_slice(&self.projection_matrix.to_raw())
        );

    }


    fn load_texture_bytes(
        &mut self, 
        data: &[u8]
    ) -> tataku::TatakuResult<tataku::TextureReference> {
        let diffuse_image = image::load_from_memory(data)
            .map_err(|e| tataku::Error::String(e.to_string()))?;

        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let (width, height) = diffuse_image.dimensions();

        self.load_texture_rgba(&diffuse_rgba, [width, height])
    }

    fn load_texture_rgba(
        &mut self,
        data: &[u8],
        [width, height]: [u32; 2]
    ) -> tataku::TatakuResult<tataku::TextureReference> {
        let Some(info) = self.atlas.try_insert(width, height)
        else { return Err(tataku::Error::String("no space in atlas".to_owned())); };

        if info.is_empty() { return Ok(info) }

        let mut data = Cow::Borrowed(data);

        if self.config.format.remove_srgb_suffix() != wgpu::TextureFormat::Rgba8Unorm {
            // cast to bgra
            data = data
                .chunks_exact(4)
                .flat_map(|b| cast_from_rgba_bytes(b, self.config.format))
                .collect::<Vec<_>>()
                .into();
        }

        let padded_width = width + 2 * tataku::ATLAS_PADDING;
        let padded_height = height + 2 * tataku::ATLAS_PADDING;

        let top_bottom_padding = || (0..padded_width * tataku::ATLAS_PADDING * 4).map(|_| 0u8);
        let left_right_padding = || (0..tataku::ATLAS_PADDING * 4).map(|_| 0u8);

        let data = top_bottom_padding()
            .chain(
                data.chunks_exact(width as usize * 4)
                    .flat_map(|data| left_right_padding().chain(data.iter().copied()).chain(left_right_padding()))
            )
            .chain(top_bottom_padding())
            .collect::<Vec<_>>();

        let texture_size = wgpu::Extent3d {
            width: padded_width,
            height: padded_height,
            depth_or_array_layers: 1,
        };

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas_texture.textures
                    .get(info.layer as usize)
                    .unwrap()
                    .0,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: info.x.saturating_sub(tataku::ATLAS_PADDING),
                    y: info.y.saturating_sub(tataku::ATLAS_PADDING),
                    z: 0
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * padded_width),
                rows_per_image: None,
            },
            texture_size,
        );

        Ok(info)
    }

    fn free_tex(
        &mut self, 
        tex: tataku::TextureReference, 
        defer_until_next_draw: bool
    ) {
        if tex.is_empty() { return }

        if defer_until_next_draw {
            self.deferred_free_textures.push(tex);
            return;
        }

        // remove from texture atlas
        self.atlas.remove_entry(tex);
    }

    fn screenshot(&mut self, callback: graphics::ScreenshotCallback) {
        self.screenshot_pending = Some(Box::new(callback));
    }

    fn begin_render(&mut self) {
        // if self.last_drawn is not None at this point, something went wrong
        assert!(self.current_render_buffer.is_none());

        let mut standard_buffers = Vec::new();
        let mut slider_buffers = Vec::new();
        let mut flashlight_buffers = Vec::new();
        let mut gaussian_blur_buffers = Vec::new();
        let mut box_blur_buffers = Vec::new();
        #[cfg(feature="vello")]
        let mut vello_buffers = Vec::new();

        for i in self.completed_buffers.take() {
            match i {
                RenderBufferType::Standard(v) => standard_buffers.push(v),
                RenderBufferType::Slider(s) => slider_buffers.push(s),
                RenderBufferType::Flashlight(f) => flashlight_buffers.push(f),
                RenderBufferType::GaussianBlur(f) => gaussian_blur_buffers.push(f),
                RenderBufferType::BoxBlur(f) => box_blur_buffers.push(f),
                #[cfg(feature="vello")]
                RenderBufferType::Vello(b) => vello_buffers.push(b),
            }
        }

        for i in self.buffer_queues.values_mut() {
            match &mut **i {
                RenderBufferQueueType::Slider(s) => s.begin(slider_buffers.take()),
                RenderBufferQueueType::Standard(v) => v.begin(standard_buffers.take()),
                RenderBufferQueueType::Flashlight(f) => f.begin(flashlight_buffers.take()),
                RenderBufferQueueType::GaussianBlur(f) => f.begin(gaussian_blur_buffers.take()),
                RenderBufferQueueType::BoxBlur(f) => f.begin(box_blur_buffers.take()),

                #[cfg(feature="vello")]
                RenderBufferQueueType::Vello(b) => b.begin(vello_buffers.take()),
            }
        }
    }

    fn end_render(&mut self) {
        let Some(mut last_queue) = self.current_render_buffer.take()
        else { return };

        if let Some(b) = last_queue.end(&self.queue) {
            self.completed_buffers.push(b);
        }

        self.buffer_queues.insert(last_queue.pipeline_type(), last_queue);
    }

    fn present(&mut self) -> tataku::TatakuResult<()> {
        self.render_current_surface()
            .map_err(|e| tataku::Error::String(e.to_string()))
    }


    // particle engine stuff
    fn add_emitter(&mut self, emitter: tataku::EmitterReference) {
        self.particle_system.add(emitter);
    }

    fn update_emitters(&mut self) {
        self.particle_system.update(&self.device, &self.queue);
    }

    fn with_renderer(&mut self, draw: &dyn Fn(&mut dyn graphics::DrawEngine)) {
        #[cfg(feature="vello")]
        if self.vello_pipeline.is_some() {
            let mut engine = shaders::vello::RenderEngine::new(
                self
            );

            draw(&mut engine);
            return;
        }

        draw(self);
    }
}

impl graphics::DrawEngine for WgpuEngine<'_> {
    fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push_scissor(scissor);
    }
    fn pop_scissor(&mut self) {
        self.scissors.pop_scissor();
    }

    /// draw an arc with the center at 0,0
    fn draw_arc(
        &mut self,
        start: f32,
        end: f32,
        radius: f32,
        color: tataku::Color,
        _border: Option<tataku::Border>,
        resolution: u32,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
    ) {
        let n = resolution;
        let x = -radius;
        let y = -radius;
        let w = 2.0 * radius;
        let h = 2.0 * radius;

        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);

        let mut path = LyonPath::builder();
        for i in 0..=n {
            let angle = f32::lerp(start, end, i as f32 / n as f32);
            let p = Point::new(
                cx + angle.cos() * cw,
                cy + angle.sin() * ch
            );

            if i == 0 {
                path.begin(p);
            } else {
                path.line_to(p);
            }
        }
        path.end(false);
        let path = path.build();

        self.tessellate_path(&path, color, None, transform, blend_mode);
    }

    fn draw_circle(
        &mut self,
        radius: f32,
        color: tataku::Color,
        border: Option<tataku::Border>,
        resolution: u32,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode
    ) {
        let n = resolution;
        let x = -radius;
        let y = -radius;
        let w = 2.0 * radius;
        let h = 2.0 * radius;
        use std::f32::consts::PI;

        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);
        let points = (0..n).map(|i| {
            let angle = i as f32 / n as f32 * (PI * 2.0);
            tataku::Vector2::new(
                cx + angle.cos() * cw,
                cy + angle.sin() * ch
            )
        }).collect::<Vec<_>>();

        // fill
        if color.a > 0 {
            self.tessellate_polygon(
                &points,
                color,
                None,
                transform,
                blend_mode
            );
        }

        // border
        if let Some(border) = border.filter(|b| b.color.a > 0) {
            self.tessellate_polygon(
                &points,
                border.color,
                Some(border.width),
                transform,
                blend_mode
            );
        }

    }

    fn draw_line(
        &mut self,
        p2: tataku::Vector2,
        thickness: f32,
        color: tataku::Color,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
    ) {
        let p1 = tataku::Vector2::ZERO;

        let n = p2 - p1;
        let n = tataku::Vector2::new(-n.y, n.x).normalize() * thickness;

        let n0 = p1 + n;
        let n1 = p2 + n;
        let n2 = p1 - n;
        let n3 = p2 - n;

        let quad = [ n0, n2, n1, n3 ];
        self.reserve_quad(quad, color, transform, blend_mode);
    }

    /// rect is [x,y,w,h]
    fn draw_rect(
        &mut self,
        rect: [f32; 4],
        border: Option<tataku::Border>,
        shape: graphics::Shape,
        color: tataku::Color,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
    ) {
        // for some reason something gets set to infinity on screen resize and panics the tesselator, this prevents the panic
        if rect.iter().any(|n| !n.is_normal() && *n != 0.0) { return }

        let [x, y, w, h] = rect;
        let rect = Box2D::new(
            Point::new(x, y),
            Point::new(x + w, y + h)
        );


        use lyon_tessellation::path::{ Path, Winding };
        let mut path = Path::builder();
        match shape {
            graphics::Shape::Square => path.add_rectangle(&rect, Winding::Positive),
            graphics::Shape::Round(radius) => path.add_rounded_rectangle(
                &rect,
                &BorderRadii::new(radius),
                Winding::Positive
            ),

            graphics::Shape::RoundSep([
                top_left,
                top_right,
                bottom_left,
                bottom_right
            ]) => path.add_rounded_rectangle(
                &rect,
                &BorderRadii {
                    top_left,
                    top_right,
                    bottom_left,
                    bottom_right
                },
                Winding::Positive
            ),
        }
        let path = path.build();

        // fill
        if color.a > 0 {
            self.tessellate_path(
                &path,
                color,
                None,
                transform,
                blend_mode
            );
        }

        // border
        if let Some(border) = border.filter(|b| b.color.a > 0) {
            self.tessellate_path(
                &path,
                border.color,
                Some(border.width),
                transform,
                blend_mode
            );
        }
    }

    fn draw_tex(
        &mut self,
        tex: graphics::TextureDraw,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
    ) {
        self.reserve_tex_quad(
            tex.tex,
            [0.0, 0.0, tex.tex.width as f32, tex.tex.height as f32],
            tex.color,
            tex.flip.flip_h(),
            tex.flip.flip_v(),
            transform,
            blend_mode
        );
    }


    fn draw_slider(
        &mut self,
        quad: [tataku::Vector2; 4],
        transform: tataku::Matrix,

        mut slider_data: tataku::SliderData,
        slider_grids: Vec<tataku::GridCell>,
        grid_cells: Vec<u32>,
        line_segments: Vec<tataku::LineSegment>
    ) {
        let Some(mut reserved) = self.reserve_slider(
            slider_grids.len() as u64,
            grid_cells.len() as u64,
            line_segments.len() as u64
        ) else {
            // self.dump_atlas("debug/atlas");
            // panic!("couldnt reserve slider?");
            return
        };

        let vertices = quad
            .into_iter()
            .map(|p| shaders::slider::Vertex {
                position: transform.mul_v2(p).into(),
                slider_index: reserved.slider_index,
            })
            .collect::<Vec<_>>();

        let offset = reserved.idx_offset as u32;
        slider_data.grid_index += reserved.slider_grid_offset;
        #[allow(clippy::identity_op, reason = "0 + offset is nice")]
        reserved.copy_in(
            &vertices,
            &[
                0 + offset,
                2 + offset,
                1 + offset,

                1 + offset,
                2 + offset,
                3 + offset,
            ],
            slider_data.into(),

            &slider_grids.into_iter()
                .map(|mut a| {
                    a.index += reserved.grid_cell_offset;
                    a.into()
                })
                .collect::<Vec<_>>(),

            &grid_cells
                .into_iter()
                .map(|i|i + reserved.line_segment_offset)
                .collect::<Vec<_>>(),

            &line_segments
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<_>>()
        );
    }

    fn draw_flashlight(
        &mut self,
        quad: [tataku::Vector2; 4],
        transform: tataku::Matrix,
        flashlight_data: tataku::FlashlightData
    ) {
        let Some(mut reserved) = self.reserve_flashlight()
        else { return };

        let vertices = quad.into_iter()
            .map(|p| shaders::flashlight::Vertex {
                position: transform.mul_v2(p).into(),
                flashlight_index: reserved.flashlight_index,
            })
            .collect::<Vec<_>>();

        reserved.copy_in(&vertices, flashlight_data);
    }


    fn draw_box_blur(
        &mut self,
        bounds: tataku::Bounds,
        size: u32,
    ) {
        let Some(mut reserve) = self.reserve_box_blur()
        else { return };

        let params = shaders::box_blur::Params::new(
            bounds.pos.x.max(0.0) as u32,
            bounds.pos.y.max(0.0) as u32,
            bounds.size.x.max(0.0) as u32,
            bounds.size.y.max(0.0) as u32,
            size,
        );

        reserve.copy_in(params);
    }

    fn draw_gaussian_blur(
        &mut self,
        bounds: tataku::Bounds,
        sigma: f32,
        _rounds: u32,
    ) {
        let Some(mut reserve) = self.reserve_gaussian_blur()
        else { return };

        let params = shaders::gaussian_blur::Params::new(
            bounds.pos.x.floor() as u32,
            bounds.pos.y.floor() as u32,
            bounds.size.x.ceil() as u32,
            bounds.size.y.ceil() as u32,
            sigma,
        );

        reserve.copy_in(params);
    }

    fn draw_text(
        &mut self,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,
        layout: &parley::Layout<tataku_client_common::prelude::Color>,
    ) {
        use parley::swash::{
            FontRef,
            scale::{
                Render, Source, StrikeWith,
            }
        };

        let runs = layout.lines()
            .flat_map(|line| line.items())
            .flat_map(|item| match item {
                parley::PositionedLayoutItem::GlyphRun(glyph_run) => Some(glyph_run),
                parley::PositionedLayoutItem::InlineBox(_) => None,
            });

        let mut render = Render::new(&[
            // Color outline with the first palette
            Source::ColorOutline(0),
            // Color bitmap with best fit selection mode
            Source::ColorBitmap(StrikeWith::BestFit),
            // Standard scalable outline
            Source::Outline,
        ]);

        let mut glyphs = Vec::new();
        struct Glyph {
            pos: tataku::Vector2,
            color: tataku::Color,
            image: parley::swash::scale::image::Image,
        }

        for run in runs {
            let inner = run.run();
            let font = inner.font();
            let size = inner.font_size();
            let color = run.style().brush;

            let mut scaler = self.font_scale_context.builder(FontRef::from_index(
                font.data.data(), 
                font.index as usize
            ).unwrap())
                .size(size)
                .build();

            for glyph in run.positioned_glyphs() {
                let offset = [
                    glyph.x.fract(),
                    0.0, // quantize = true
                ];

                render.offset(offset.into());

                let Some(image) = render.render(&mut scaler, glyph.id) 
                else { continue; };

                let x = glyph.x.floor() as i32;
                let y = glyph.y.floor() as i32;

                // convert from bottom-left to top-left image
                let x = x + image.placement.left;
                let y = y - image.placement.top;

                glyphs.push(Glyph {
                    pos: tataku::Vector2::new(x as f32, y as f32),
                    color,
                    image,
                });

            }
        }

        for g in glyphs {
            let image = g.image;

            let size = [image.placement.width, image.placement.height];

            let data = image.data.iter()
                .map(|&alpha| g.color.alpha8(alpha))
                .flat_map(|color| [color.r, color.g, color.b, color.a])
                .collect::<Vec<_>>();

            let tex = self.load_texture_rgba(&data, size).unwrap();

            self.draw_tex(
                graphics::TextureDraw::new(
                    &tex,
                    tataku::Color::WHITE,
                ),
                transform.trans(g.pos),
                blend_mode,
            );

            self.free_tex(tex, true);
        }

    }
}



struct VsyncUtils;
impl VsyncUtils {
    fn map_to_vsync(present_mode: wgpu::PresentMode) -> tataku::Vsync {
        match present_mode {
            wgpu::PresentMode::AutoVsync => tataku::Vsync::AutoVsync,
            wgpu::PresentMode::AutoNoVsync => tataku::Vsync::AutoNoVsync,
            wgpu::PresentMode::Fifo => tataku::Vsync::Fifo,
            wgpu::PresentMode::FifoRelaxed => tataku::Vsync::FifoRelaxed,
            wgpu::PresentMode::Immediate => tataku::Vsync::Immediate,
            wgpu::PresentMode::Mailbox => tataku::Vsync::Mailbox,
        }
    }
    fn map_from_vsync(vsync: tataku::Vsync) -> wgpu::PresentMode {
        match vsync {
            tataku::Vsync::AutoVsync => wgpu::PresentMode::AutoVsync,
            tataku::Vsync::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
            tataku::Vsync::Fifo => wgpu::PresentMode::Fifo,
            tataku::Vsync::FifoRelaxed => wgpu::PresentMode::FifoRelaxed,
            tataku::Vsync::Immediate => wgpu::PresentMode::Immediate,
            tataku::Vsync::Mailbox => wgpu::PresentMode::Mailbox,
        }
    }
}


fn cast_from_rgba_bytes(bytes: &[u8], format: wgpu::TextureFormat) -> [u8; 4] {
    // incoming is rgba8
    #[allow(clippy::get_first, reason = "get(0) keeps things lined up here")]
    let r = bytes.get(0).copied().unwrap_or_default();
    let g = bytes.get(1).copied().unwrap_or_default();
    let b = bytes.get(2).copied().unwrap_or_default();
    let a = bytes.get(3).copied().unwrap_or_default();

    match format {
        // pretend this is all it can be for now
        wgpu::TextureFormat::Bgra8Unorm
        | wgpu::TextureFormat::Bgra8UnormSrgb => [b, g, r, a],

        // just default to rgba otherwise and cry if its not
        _ => [r, g, b, a]
    }

}

fn cast_to_rgba_bytes(bytes: &[u8], _format: wgpu::TextureFormat) -> [u8; 4] {
    // pretend incoming is bgra8
    #[allow(clippy::get_first, reason = "get(0) keeps things lined up here")]
    let b = bytes.get(0).copied().unwrap_or_default();
    let g = bytes.get(1).copied().unwrap_or_default();
    let r = bytes.get(2).copied().unwrap_or_default();
    let a = bytes.get(3).copied().unwrap_or_default();

    [r, g, b, a]

    // match format {
    //     // pretend this is all it can be for now
    //     TextureFormat::Bgra8Unorm
    //     | TextureFormat::Bgra8UnormSrgb => [b, g, r, a],
    //     TextureFormat::Rgba8Unorm

    //     // just default to rgba otherwise and cry if its not
    //     _ => [r, g, b, a]
    // }

}

#[derive(Copy, Clone)]
pub(crate) enum WgpuPipeline<'a> {
    None,
    Render(&'a wgpu::RenderPipeline),
    Compute(&'a wgpu::ComputePipeline),
}
impl WgpuPipeline<'_> {
    pub fn get_bind_group_layout(&self, index: u32) -> wgpu::BindGroupLayout {
        match self {
            Self::None => panic!("trying to get bind group for no pipeline!"),
            Self::Render(p) => p.get_bind_group_layout(index),
            Self::Compute(p) => p.get_bind_group_layout(index),
        }
    }
}
impl<'a> From<&'a wgpu::ComputePipeline> for WgpuPipeline<'a> {
    fn from(value: &'a wgpu::ComputePipeline) -> Self {
        Self::Compute(value)
    }
}
impl<'a> From<&'a wgpu::RenderPipeline> for WgpuPipeline<'a> {
    fn from(value: &'a wgpu::RenderPipeline) -> Self {
        Self::Render(value)
    }
}
