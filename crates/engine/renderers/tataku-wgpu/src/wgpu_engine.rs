// WARNING: there is a lot of just data and setup code in this

use wgpu::Queue;
use crate::prelude::*;
use super::shaders::*;
use std::num::NonZeroU64;

use tataku_engine::prelude::*;
use tataku_graphics::prelude::*;
use wgpu::util::DeviceExt as _;
use winit::raw_window_handle::{ HasWindowHandle, HasDisplayHandle };
use tataku_client_common::prelude::Color;
use lyon_tessellation::{ 
    geom::{ Box2D, Point }, 
    path::builder::BorderRadii,
    path::Path as LyonPath,
};


// must not go past 16
const LAYER_COUNT:u32 = 12;
const MAX_DEPTH:f32 = 8192.0 * 8192.0;

/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;

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
    surface: Surface<'window>,
    device: Device,
    queue: Arc<Queue>,
    config: SurfaceConfiguration,

    pipelines: HashMap<Pipeline, RenderPipeline>,

    buffer_queues: HashMap<LastPipeline, Box<RenderBufferQueueType>>,
    completed_buffers: Vec<RenderBufferType>,
    current_render_buffer: Option<Box<RenderBufferQueueType>>,

    projection_matrix: Matrix,
    projection_matrix_buffer: Buffer,
    projection_matrix_bind_group: BindGroup,

    atlas: Atlas,
    atlas_texture: WgpuTexture,

    screenshot_pending: Option<ScreenshotCallback>,

    sampler: Sampler,
    particle_system: ParticleSystem,
    gaussian_blur_shader: RefCell<GaussianBlurShader>,
    box_blur_shader: RefCell<BoxBlurShader>,
    render_image_shader: RenderImageShader,

    scissors: ScissorManager,

    present_modes: Vec<Vsync>,
    can_blur: bool,
    blur_enabled: bool,

    intermediate_texture: Texture,
}
impl<'window> WgpuEngine<'window> {

    // Creating some of the wgpu types requires async code
    pub async fn create<W:HasWindowHandle + HasDisplayHandle + Sync>(
        window: &'window W, 
        settings: &DisplaySettings,
    ) -> Box<dyn GraphicsEngine + 'window> {
        let window_size = settings.window_size;

        // create a wgpu instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::METAL, // | Backends::GL,
            flags: InstanceFlags::empty(),
            gles_minor_version: Gles3MinorVersion::Automatic,
            dx12_shader_compiler: Dx12Compiler::default(),
        });

        // create the surface
        let surface: Surface<'window> = instance
            .create_surface(window)
            .unwrap();

        // create the adapter
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: match settings.performance_mode {
                PerformanceMode::HighPerformance => PowerPreference::HighPerformance,
                PerformanceMode::PowerSaver => PowerPreference::LowPower,
            },
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        // create device and queue
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                #[cfg(feature="texture_arrays")]
                required_features: Features::TEXTURE_BINDING_ARRAY 
                    | Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING 
                    | Features::BGRA8UNORM_STORAGE,
                #[cfg(not(feature="texture_arrays"))]
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
                label: None,
            },
            None,
        ).await.unwrap();

        let can_blur = device.features().contains(Features::BGRA8UNORM_STORAGE);
        if !can_blur { warn!("Blur unsupported on this device!"); }

        // no more comments good luck!
        let surface_caps = surface.get_capabilities(&adapter);
        let present_modes = surface_caps
            .present_modes
            .into_iter()
            .map(VsyncUtils::map_to_vsync)
            .chain([Vsync::AutoNoVsync, Vsync::AutoVsync])
            .collect();

        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT, 
            format: surface_format,
            width: window_size[0] as u32,
            height: window_size[1] as u32,
            present_mode: PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],

            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        #[cfg(feature="texture_arrays")]
        let texture_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("atlas group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: std::num::NonZeroU32::new(LAYER_COUNT),
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            }
        );

        #[cfg(not(feature="texture_arrays"))]
        let texture_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("atlas group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            }
        );

        let proj_matrix_size = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
        let projection_matrix_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("Texture/Sampler bind group layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(proj_matrix_size)
                        },
                        count: None,
                    },
                ]
            }
        );

        let window_size = Vector2::new(window_size[0], window_size[1]);
        let projection_matrix = Self::create_projection(window_size);
        let projection_matrix_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Projection Matrix Buffer"),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&projection_matrix.to_raw()),
            }
        );

        let projection_matrix_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("diffuse_bind_group"),
                layout: &projection_matrix_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::Buffer(BufferBinding {
                            buffer: &projection_matrix_buffer,
                            offset: 0,
                            size: NonZeroU64::new(proj_matrix_size)
                        }),
                    },
                ],
            }
        );

        let mut pipelines = create_standard_pipeline(
            &device, 
            &projection_matrix_bind_group_layout, 
            &texture_bind_group_layout
        );


        // create slider pipeline
        pipelines.insert(Pipeline::Slider, create_slider_pipeline(
            &device, 
            &projection_matrix_bind_group_layout
        ));

        // create flashlight pipeline
        pipelines.insert(Pipeline::Flashlight, create_flashlight_pipeline(
            &device, 
            &projection_matrix_bind_group_layout
        ));


        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let gaussian_blur_shader = GaussianBlurShader::new(&device);
        let box_blur_shader = BoxBlurShader::new(&device);
        let render_image_shader = RenderImageShader::new(
            &device, 
            &queue, 
            &projection_matrix_bind_group_layout
        );


        let atlas_size = device.limits().max_texture_dimension_2d.min(8192);
        let atlas_texture = Self::create_texture(
            &device, 
            &texture_bind_group_layout, 
            &sampler, 
            atlas_size, 
            atlas_size, 
            TextureFormat::Bgra8Unorm,
        );

        let particle_system = ParticleSystem::new(&device);

        let buffer_queues = [
            (LastPipeline::Slider, Box::new(RenderBufferQueueType::Slider(
                RenderBufferQueue::default().init(
                    &device, 
                    &pipelines[&Pipeline::Slider]
                )
            ))),
            (LastPipeline::Standard, Box::new(RenderBufferQueueType::Standard(
                RenderBufferQueue::default().init(
                    &device, 
                    &pipelines[&Pipeline::AlphaBlending]
                )
            ))),
            (LastPipeline::Flashlight, Box::new(RenderBufferQueueType::Flashlight(
                RenderBufferQueue::default().init(
                    &device, 
                    &pipelines[&Pipeline::Flashlight]
                )
            ))),
            (LastPipeline::GaussianBlur, Box::new(RenderBufferQueueType::GaussianBlur(
                RenderBufferQueue::default().init(
                    &device, 
                    &gaussian_blur_shader.pipeline
                )
            ))),
            (LastPipeline::BoxBlur, Box::new(RenderBufferQueueType::BoxBlur(
                RenderBufferQueue::default().init(
                    &device, 
                    &box_blur_shader.pipeline
                )
            ))),
        ].into_iter().collect();


        // because the swapchain texture can only have RenderAttachment (**annoy**)
        // we render to an intermediary texture, which can have blur applied and used for screenshots
        let intermediate_texture = device.create_texture(
            &TextureDescriptor { 
                label: Some("Render Texture"),
                size: Extent3d { 
                    width: config.width, 
                    height: config.height, 
                    depth_or_array_layers: 1 
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8Unorm,
                usage: TextureUsages::RENDER_ATTACHMENT 
                    | TextureUsages::COPY_SRC 
                    | TextureUsages::COPY_DST 
                    | TextureUsages::TEXTURE_BINDING,
                view_formats: &[ TextureFormat::Bgra8UnormSrgb ]
            }
        );

        Box::new(Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            pipelines,
            atlas: Atlas::new(
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
            gaussian_blur_shader: RefCell::new(gaussian_blur_shader),
            box_blur_shader: RefCell::new(box_blur_shader),
            render_image_shader,

            scissors: ScissorManager::default(),
            present_modes,
            sampler,
            can_blur,
            blur_enabled: true,
            intermediate_texture,
        })
    }

    pub fn render_current_surface(&mut self) -> Result<(), SurfaceError> {
        let swapchain = self.surface.get_current_texture()?;
        let size = swapchain.texture.size();

        // don't draw if our draw surface has no area
        if size.width == 0 || size.height == 0 { return Ok(()) }

        if self.intermediate_texture.size() != size {
            self.intermediate_texture = self.device.create_texture(
                &TextureDescriptor {
                    label: Some("Render Texture"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: self.intermediate_texture.dimension(),
                    format: self.intermediate_texture.format(),
                    usage: self.intermediate_texture.usage(),
                    view_formats: &[ TextureFormat::Bgra8UnormSrgb ]
                }
            );
        }

        
        let tex = WgpuTextureReference::new(&self.intermediate_texture);
        self.render(&RenderableSurface::new(
            &tex,
            GFX_CLEAR_COLOR, 
            Vector2::new(size.width as f32, size.height as f32), 
            true
        ))?;

        // `texture` should now have our data, with which we can use to render the surface, as well as use for screenshots
        // again though, because the swapchain texture can only be rendered to directly for some reason, we have to use a shader
        let mut encoder = self.device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let output_view = swapchain
                .texture
                .create_view(&TextureViewDescriptor::default());

            let mut render = encoder.begin_render_pass(
                &RenderPassDescriptor {
                    label: None,
                    color_attachments: &[
                        Some(RenderPassColorAttachment {
                            view: &output_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::BLACK),
                                store: StoreOp::Store,
                            }
                        })
                    ],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                }
            );
            render.set_pipeline(&self.render_image_shader.pipeline);

            let bind_group = self.device.create_bind_group(
                &BindGroupDescriptor {
                    label: None,
                    layout: &self.render_image_shader.bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Sampler(&self.sampler),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&tex.view),
                        },
                    ]
                }
            );

            self.render_image_shader.update_buffer(
                &self.queue,
                [size.width, size.height]
            );
            self.queue.submit([]);

            render.set_bind_group(
                0, 
                &bind_group, 
                &[]
            );

            render.set_bind_group(
                1, 
                &self.projection_matrix_bind_group, 
                &[]
            );

            render.set_vertex_buffer(
                0, 
                self.render_image_shader.buffer.slice(..)
            );

            render.draw(0..6, 0..1);
        }        
        self.queue.submit([encoder.finish()]);
        swapchain.present();



        // self.render(RenderableSurface::new(
        //     &view, 
        //     GFX_CLEAR_COLOR, 
        //     Vector2::new(size.width as f32, size.height as f32),
        //     false,
        // ))?;

        // let width = output.texture.width();
        // let height = output.texture.height();


        // 

        // output.present();



        if let Some(screenshot) = self.screenshot_pending.take() {
            // let texture = self.device.create_texture(&TextureDescriptor {
            //     label: Some("Screenshot Texture"),
            //     size: Extent3d {
            //         width,
            //         height,
            //         depth_or_array_layers: 1
            //     },
            //     mip_level_count: 1,
            //     sample_count: 1,
            //     dimension: TextureDimension::D2,
            //     format: TextureFormat::Bgra8UnormSrgb,
            //     usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            //     view_formats: &[]
            // });
            // let view = texture.create_view(&TextureViewDescriptor {
            //     label: Some("Screenshot Texture View"),
            //     dimension: Some(TextureViewDimension::D2),
            //     base_array_layer: 0,

            //     ..Default::default()
            // });

            // self.render(RenderableSurface::new(
            //     &view, 
            //     GFX_CLEAR_COLOR, 
            //     Vector2::new(width as f32, height as f32), 
            //     true
            // ))?;

            self.finish_screenshot(screenshot);
        }


        Ok(())
    }

    fn render(&self, renderable: &RenderableSurface) -> Result<(), SurfaceError> {
        let mut encoder = self.device.create_command_encoder(
            &CommandEncoderDescriptor { label: Some("Render Encoder") }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &renderable.texture.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(renderable.get_clear_color()),
                            store: if renderable.render_target { 
                                StoreOp::Store  // must be store for render targets to work apparently
                            } else {
                                StoreOp::Discard 
                            },
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                }
            );

            let mut current_pipeline = Pipeline::None;
            let mut current_scissor: Scissor = None;

            for i in self.completed_buffers.iter() {
                // blurs are a special case, they're compute shaders and not fragment shaders
                let pipeline = i.get_pipeline();
                if pipeline.is_blur() {
                    if !self.can_blur || !self.blur_enabled { continue }

                    // finish and submit the current render pass to free up the encoder
                    drop(render_pass);
                    self.queue.submit([encoder.finish()]);

                    // perform the blur
                    match pipeline {
                        Pipeline::GaussianBlur => {
                            let RenderBufferType::GaussianBlur(buffer) = i 
                            else { unreachable!() };

                            self.gaussian_blur_shader.borrow_mut().perform(
                                &self.device, 
                                &self.queue, 
                                renderable.texture,
                                buffer
                            );
                        }
                        Pipeline::BoxBlur => {
                            let RenderBufferType::BoxBlur(buffer) = i 
                            else { unreachable!() };

                            self.box_blur_shader.borrow_mut().perform(
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
                        &CommandEncoderDescriptor { 
                            label: Some("Render Encoder") 
                        }
                    );

                    render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &renderable.texture.view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: StoreOp::Store, // must be store for render targets to work apparently
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    current_pipeline = Pipeline::None;
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
                        current_pipeline = Pipeline::None;
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
                    IndexFormat::Uint32
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

    fn create_projection(draw_size: Vector2) -> Matrix {
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
        device: &Device, 
        layout: &BindGroupLayout, 
        sampler: &Sampler, 
        width: u32, 
        height: u32, 
        format: TextureFormat,
    ) -> WgpuTexture {
        let texture_size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let textures = (0..LAYER_COUNT).map(|_| {
            let texture = device.create_texture(
                &TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    // Most images are stored using sRGB so we need to reflect that here.
                    format, //TextureFormat::Rgba8UnormSrgb,
                    // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                    // COPY_DST means that we want to copy data to this texture
                    usage: TextureUsages::TEXTURE_BINDING 
                        | TextureUsages::COPY_DST 
                        | TextureUsages::COPY_SRC 
                        | TextureUsages::RENDER_ATTACHMENT,
                    label: Some("atlas_texture"),
                    view_formats: &[],
                }
            );

            let view = texture.create_view(&TextureViewDescriptor {
                label: Some("atlas_texture_view"),
                dimension: Some(TextureViewDimension::D2),
                base_array_layer: 0,

                ..Default::default()
            });

            (texture, view)
        })
        .collect::<Vec<_>>();


        #[cfg(feature="texture_arrays")]
        let view_list = textures.iter()
            .map(|a| &a.1)
            .collect::<Vec<_>>();

        #[cfg(feature="texture_arrays")]
        let bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("texture array bind group"),
                layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureViewArray(&view_list),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(sampler),
                    }
                ],
            }
        );

        #[cfg(not(feature="texture_arrays"))]
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture array bind group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&textures[0].1),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&textures[1].1),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&textures[2].1),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(&textures[3].1),
                },
            ],
        });
        
        WgpuTexture {
            textures: Arc::new(textures),
            bind_group
        }
    }

    fn read_texture(&self, texture: &Texture) -> (Vec<u8>, [u32;2]) {
        let (w, h) = (texture.width(), texture.height());

        let fuck = (w * 4)
            .div_ceil(COPY_BYTES_PER_ROW_ALIGNMENT) 
            * COPY_BYTES_PER_ROW_ALIGNMENT;

        let size = (fuck * h) as u64; //(w * h * 4) as u64;
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Texture Reading Buffer"),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            size,
            mapped_at_creation: false,
        });

        let tex_buffer = ImageCopyBuffer {
            buffer: &buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(fuck),
                rows_per_image: None
            }
        };

        let mut encoder = self.device.create_command_encoder(
            &CommandEncoderDescriptor { label: Some("Texture reading encoder") }
        );
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(), 
            tex_buffer, 
            texture.size()
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        slice.map_async(MapMode::Read, |_| {});
        let index = self.queue.submit(None);
        self.device.poll(MaintainBase::WaitForSubmissionIndex(index));
    
        let data = slice
            .get_mapped_range()
            .chunks_exact(4)
            .flat_map(|b| cast_to_rgba_bytes(b, texture.format()))
            .collect();

        (data, [fuck / 4, h])
    }

    fn finish_screenshot(&self, callback: ScreenshotCallback) {
        let (data, size) = self.read_texture(&self.intermediate_texture);

        callback((data, size));
    }
}


// render code
impl WgpuEngine<'_> {
    fn dump_last_drawn(&mut self) {
        let Some(mut last_drawn) = self
            .current_render_buffer.take()
        else { return };

        let gaussian_blur = self.gaussian_blur_shader.borrow();
        let box_blur = self.box_blur_shader.borrow();

        let pipeline = match last_drawn.draw_type().as_blendmode() {
            Pipeline::GaussianBlur => WgpuPipeline::Compute(&gaussian_blur.pipeline),
            Pipeline::BoxBlur => WgpuPipeline::Compute(&box_blur.pipeline),
            other => WgpuPipeline::Render(&self.pipelines[&other]),
        };
        if let Some(b) = last_drawn.dump_and_next(
            &self.queue, 
            &self.device, 
            pipeline
        ) { 
            self.completed_buffers.push(b); 
        };

        self.buffer_queues.insert(last_drawn.draw_type(), last_drawn);
    }

    fn check_dump_and_next(&mut self, to_draw: LastPipeline) {
        if let Some(last_drawn) = &self.current_render_buffer {
            if last_drawn.draw_type() == to_draw { return }
        }

        self.dump_last_drawn();
        self.current_render_buffer = Some(self.buffer_queues
            .remove(&to_draw)
            .unwrap_or_else(|| panic!("buffer queue did not have a queue for type {to_draw:?}. Did you forget to create a buffer queue for it?"))
        );
    }

    /// returns reserve data
    fn reserve_standard(
        &mut self,
        vtx_count: u64,
        idx_count: u64,
        blend_mode: Pipeline
    ) -> Option<StandardReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastPipeline::Standard);

        let vertex_buffer_queue = get_render_buffer!(self, Standard);

        let mut recording_buffer = vertex_buffer_queue
            .recording_buffer()
            .expect("didnt get vertex recording buffer");

        if !( // blend mode check
            recording_buffer.blend_mode == blend_mode 
            || recording_buffer.blend_mode == Pipeline::None
        )
        || !( // scissor check
            recording_buffer.scissor == Some(scissor) 
            || recording_buffer.scissor.is_none()
        )
        || recording_buffer.used_vertices + vtx_count > StandardBuffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > StandardBuffer::IDX_PER_BUF {
            let pipeline = WgpuPipeline::Render(
                &self.pipelines[&blend_mode]
            );
            
            if let Some(b) = vertex_buffer_queue.dump_and_next(
                &self.queue, 
                &self.device, 
                pipeline
            ) {
                self.completed_buffers.push(RenderBufferType::Standard(b));
            }

            recording_buffer = vertex_buffer_queue.recording_buffer()?;
            recording_buffer.blend_mode = blend_mode;
            recording_buffer.scissor = Some(scissor);
        }
        if recording_buffer.blend_mode == Pipeline::None {
            recording_buffer.blend_mode = blend_mode;
        }
        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used_indices += idx_count;
        recording_buffer.used_vertices += vtx_count;

        let used_vertices = recording_buffer.used_vertices;
        let used_indices = recording_buffer.used_indices;

        let cache = &mut vertex_buffer_queue.cpu_cache;
        Some(StandardReserveData {
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
        tex: &TextureReference,
        rect: [f32; 4],
        color: Color,
        h_flip: bool,
        v_flip: bool,
        transform: Matrix,
        blend_mode: Pipeline,
    ) {
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
                StandardVertex {
                    position: transform.mul_v2(Vector2::new(x, y)).into(),
                    tex_coords: tl,
                    tex_index,
                    color,
                },
                StandardVertex {
                    position: transform.mul_v2(Vector2::new(x+w, y)).into(),
                    tex_coords: tr,
                    tex_index,
                    color,
                },
                StandardVertex {
                    position: transform.mul_v2(Vector2::new(x, y+h)).into(),
                    tex_coords: bl,
                    tex_index,
                    color,
                },
                StandardVertex {
                    position: transform.mul_v2(Vector2::new(x+w, y+h)).into(),
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
        quad: [Vector2; 4],
        color: Color,
        transform: Matrix,
        blend_mode: Pipeline,
    ) {
        let Some(mut reserved) = self.reserve_standard(
            4, 
            6, 
            blend_mode
        ) else { return };
        let color = color.into();

        let vertices = quad.into_iter()
            .map(|p| StandardVertex {
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

    fn reserve_slider(
        &mut self,
        slider_grid_count: u64,
        grid_cell_count: u64,
        line_segment_count: u64,
    ) -> Option<SliderReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastPipeline::Slider);

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

        if slider_grid_count > SLIDER_GRID_COUNT
        || grid_cell_count > GRID_CELL_COUNT
        || line_segment_count > LINE_SEGMENT_COUNT {
            return None
        }

        if !scissor_check
        || recording_buffer.used_vertices + vtx_count > SliderRenderBuffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > SliderRenderBuffer::IDX_PER_BUF
        || recording_buffer.used_slider_data + 1 > EXPECTED_SLIDER_COUNT
        || recording_buffer.used_slider_grids + slider_grid_count > SLIDER_GRID_COUNT
        || recording_buffer.used_grid_cells + grid_cell_count > GRID_CELL_COUNT
        || recording_buffer.used_line_segments + line_segment_count > LINE_SEGMENT_COUNT
        {
            let pipeline = WgpuPipeline::Render(
                &self.pipelines[&Pipeline::Slider]
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
        Some(SliderReserveData {
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


    fn reserve_flashlight(
        &mut self,
    ) -> Option<FlashlightReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastPipeline::Flashlight);

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
        || recording_buffer.used_vertices + vtx_count > SliderRenderBuffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > SliderRenderBuffer::IDX_PER_BUF
        {
            let pipeline = WgpuPipeline::Render(
                &self.pipelines[&Pipeline::Flashlight]
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
        Some(FlashlightReserveData {
            vtx: &mut cache.cpu_vtx[(used_vertices - vtx_count) as usize .. used_vertices as usize],
            idx: &mut cache.cpu_idx[(used_indices - idx_count) as usize .. used_indices as usize],
            flashlight_data: &mut cache.cpu_flashlights[flashlight_index as usize],

            idx_offset: used_vertices - vtx_count,
            flashlight_index: flashlight_index as u32,
        })
    }

    fn reserve_gaussian_blur(
        &mut self,
    ) -> Option<GaussianBlurReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastPipeline::GaussianBlur);

        let buffer_queue = get_render_buffer!(self, GaussianBlur);

        let mut recording_buffer = buffer_queue.recording_buffer()
            .expect("didnt get blur recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor) 
            || recording_buffer.scissor.is_none();

        if !scissor_check
            || recording_buffer.used + 1 > GaussianBlurBuffer::VTX_PER_BUF
        {
            let blur = self.gaussian_blur_shader.borrow();
            let pipeline = WgpuPipeline::Compute(&blur.pipeline);
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue, 
                &self.device, 
                pipeline
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
        Some(GaussianBlurReserveData {
            data: &mut cache.cpu_blurs[index as usize],
            _blur_index: index as u32,
        })
    }
    
    fn reserve_box_blur(
        &mut self,
    ) -> Option<BoxBlurReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastPipeline::BoxBlur);

        let buffer_queue = get_render_buffer!(self, BoxBlur);

        let mut recording_buffer = buffer_queue.recording_buffer()
            .expect("didnt get blur recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor) 
            || recording_buffer.scissor.is_none();

        if !scissor_check || recording_buffer.used {
            let blur = self.box_blur_shader.borrow();
            let pipeline = WgpuPipeline::Compute(&blur.pipeline);
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue, 
                &self.device, 
                pipeline
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
        Some(BoxBlurReserveData {
            data: &mut cache.cpu_blurs[0],
        })
    }
    

}


// draw helpers
impl WgpuEngine<'_> {
    pub(crate) fn map_blend_mode(blend_mode: Pipeline) -> BlendState {
        match blend_mode {
            Pipeline::AlphaBlending => BlendState::ALPHA_BLENDING,
            Pipeline::AlphaOverwrite => BlendState::REPLACE,
            Pipeline::PremultipliedAlpha => BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            Pipeline::AdditiveBlending => BlendState {
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
            Pipeline::OsuAdditiveBlending => BlendState {
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
            Pipeline::SourceAlphaBlending => BlendState {
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

            Pipeline::None
            | Pipeline::BoxBlur
            | Pipeline::GaussianBlur
            | Pipeline::Slider
            | Pipeline::Flashlight => unimplemented!("nope")
        }
    }

    fn tessellate_polygon(
        &mut self, 
        polygon: &[Vector2], 
        color: Color, 
        border: Option<f32>, 
        transform: Matrix, 
        blend_mode: Pipeline
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
        color: Color, 
        border: Option<f32>, 
        transform: Matrix, 
        blend_mode: Pipeline
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
                StandardVertex {
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



impl GraphicsEngine for WgpuEngine<'_> {
    fn resize(
        &mut self, 
        [width, height]: [u32; 2]
    ) {
        if width == 0 || height == 0 { return }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let window_size = Vector2::new(width as f32, height as f32);
        self.projection_matrix = Self::create_projection(window_size);
        self.queue.write_buffer(
            &self.projection_matrix_buffer, 
            0, 
            bytemuck::cast_slice(&self.projection_matrix.to_raw())
        );
    }

    fn set_vsync(&mut self, vsync: Vsync) {
        self.config.present_mode = VsyncUtils::map_from_vsync(
            vsync.to_okay(&self.present_modes)
        );
        self.surface.configure(&self.device, &self.config);
    }

    fn vsync_modes(&self) -> Vec<Vsync> {
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
        clear_color: Color, 
        do_render: RenderTargetDraw
    ) -> Option<RenderTarget> {
        // find space in the render target atlas
        let atlased = self.atlas.try_insert(width, height)?;

        // create a projection and render target
        let projection = Self::create_projection(
            Vector2::new(width as f32, height as f32)
        );

        let target = RenderTarget {
            width,
            height,
            projection,
            clear_color,
            image: Image::new(
                Vector2::ZERO,
                Arc::new(atlased),
                Vector2::ONE
            ),
        };

        // queue rendering the data to it
        self.update_render_target(target.clone(), do_render);

        // return the new render target
        Some(target)
    }
    fn update_render_target(
        &mut self, 
        target: RenderTarget, 
        do_render: RenderTargetDraw
    ) {
        // get the texture this target was written to
        let textures = self.atlas_texture.textures.clone();

        if !Bounds::new(Vector2::ZERO, target.image.size()).has_area() {
            return
        }

        let Some((atlas_tex, _)) = textures.get(target.image.tex.layer as usize) 
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
            &TextureDescriptor {
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8Unorm,
                usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
                label: Some("render_target_temp_tex"),
                view_formats: &[TextureFormat::Bgra8UnormSrgb],
            }
        );

        // create renderable surface
        let tex = WgpuTextureReference::new(&texture);
        let renderable = RenderableSurface::new(
            &tex,
            target.clear_color, 
            Vector2::new(width as f32, height as f32),
            true
        );

        // clear buffers
        self.begin_render();

        // fill buffers
        let transform = Matrix::identity();
        do_render(self, transform);

        // finish up
        self.end_render();

        // perform render
        if let Err(e) = self.render(&renderable) {
            error!("Error rendering render target: {e:?}");
        }


        // copy render to atlas
        let mut encoder = self.device.create_command_encoder(
            &CommandEncoderDescriptor { 
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


    fn load_texture_bytes(&mut self, data: &[u8]) -> TatakuResult<TextureReference> {
        let diffuse_image = image::load_from_memory(data)
            .map_err(|e| TatakuError::String(e.to_string()))?;

        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let (width, height) = diffuse_image.dimensions();

        self.load_texture_rgba(&diffuse_rgba, [width, height])
    }

    fn load_texture_rgba(
        &mut self, 
        data: &[u8], 
        [width, height]: [u32; 2]
    ) -> TatakuResult<TextureReference> {
        let Some(info) = self.atlas.try_insert(width, height) 
        else { return Err(TatakuError::String("no space in atlas".to_owned())); };

        if info.is_empty() { return Ok(info) }

        // cast to bgra
        let data = data
            .chunks_exact(4)
            .flat_map(|b| cast_from_rgba_bytes(b, self.config.format))
            .collect::<Vec<_>>()
        ;

        let texture_size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.atlas_texture.textures
                    .get(info.layer as usize)
                    .unwrap()
                    .0,
                mip_level: 0,
                origin: Origin3d {
                    x: info.x,
                    y: info.y,
                    z: 0
                },
                aspect: TextureAspect::All,
            },
            &data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        Ok(info)
    }

    fn free_tex(&mut self, tex: TextureReference) {
        if tex.is_empty() { return }

        // write empty data to where the texture was
        // this should remove the weird border when the atlas space is reused
        let width = tex.width + ATLAS_PADDING * 2;
        let height = tex.height + ATLAS_PADDING * 2;
        // empty pixels
        let data = vec![0u8; (width * height * 4) as usize];

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            ImageCopyTexture {
                texture: &self.atlas_texture.textures
                    .get(tex.layer as usize)
                    .unwrap()
                    .0,
                mip_level: 0,
                origin: Origin3d {
                    x: tex.x - ATLAS_PADDING,
                    y: tex.y - ATLAS_PADDING,
                    z: 0
                },
                aspect: TextureAspect::All,
            },
            // The actual pixel data
            &data,
            // The layout of the texture
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            }
        );

        // remove from texture atlas
        self.atlas.remove_entry(tex);
    }

    fn screenshot(&mut self, callback: ScreenshotCallback) {
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

        for i in self.completed_buffers.take() {
            match i {
                RenderBufferType::Standard(v) => standard_buffers.push(v),
                RenderBufferType::Slider(s) => slider_buffers.push(s),
                RenderBufferType::Flashlight(f) => flashlight_buffers.push(f),
                RenderBufferType::GaussianBlur(f) => gaussian_blur_buffers.push(f),
                RenderBufferType::BoxBlur(f) => box_blur_buffers.push(f),
            }
        }

        for i in self.buffer_queues.values_mut() {
            match &mut **i {
                RenderBufferQueueType::Slider(s) => s.begin(slider_buffers.take()),
                RenderBufferQueueType::Standard(v) => v.begin(standard_buffers.take()),
                RenderBufferQueueType::Flashlight(f) => f.begin(flashlight_buffers.take()),
                RenderBufferQueueType::GaussianBlur(f) => f.begin(gaussian_blur_buffers.take()),
                RenderBufferQueueType::BoxBlur(f) => f.begin(box_blur_buffers.take()),
            }
        }
    }

    fn end_render(&mut self) {
        let Some(mut last_queue) = self.current_render_buffer.take() 
        else { return };

        if let Some(b) = last_queue.end(&self.queue) {
            self.completed_buffers.push(b);
        }

        self.buffer_queues.insert(last_queue.draw_type(), last_queue);
        
    }

    fn present(&mut self) -> TatakuResult<()> {
        self.render_current_surface()
            .map_err(|e| TatakuError::String(e.to_string()))
    }


    fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push_scissor(scissor);
    }
    fn pop_scissor(&mut self) {
        self.scissors.pop_scissor();
    }


    fn load_font_data(&mut self, font: ActualFont, font_size: FontSize) {
        debug!("Loading font {} with size {font_size:?}", font.name);
        let mut characters = font.characters.write();

        for (&char, _) in font.font.chars() {
            // generate glyph data
            let (metrics, bitmap) = font.font.rasterize(char, font_size.f32());

            // bitmap is a vec of grayscale pixels
            // we need to turn that into rgba bytes
            let data = bitmap
                .into_iter()
                .flat_map(|gray| [255,255,255, gray])
                .collect::<Vec<_>>();

            let Ok(texture) = self.load_texture_rgba(
                &data, 
                [metrics.width as u32, metrics.height as u32]
            ) else { panic!("eve broke fonts") };

            let char_data = CharData { texture, metrics };
            characters.insert((font_size.u32(), char), char_data);
        }

        // let the font know the size been loaded
        font.loaded_sizes.write().insert(font_size.u32());
    }

    // draw helpers

    /// draw an arc with the center at 0,0
    fn draw_arc(
        &mut self, 
        start: f32, 
        end: f32, 
        radius: f32, 
        color: Color, 
        resolution: u32, 
        transform: Matrix, 
        blend_mode: Pipeline,
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
        color: Color, 
        border: Option<Border>, 
        resolution: u32, 
        transform: Matrix, 
        blend_mode: Pipeline
    ) {
        let n = resolution;
        let x = -radius;
        let y = -radius;
        let w = 2.0 * radius;
        let h = 2.0 * radius;

        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);
        let points = (0..n).map(|i| {
            let angle = i as f32 / n as f32 * (PI * 2.0);
            Vector2::new(
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
        p2: Vector2, 
        thickness: f32, 
        color: Color, 
        transform: Matrix, 
        blend_mode: Pipeline,
    ) {
        let p1 = Vector2::ZERO;

        let n = p2 - p1;
        let n = Vector2::new(-n.y, n.x).normalize() * thickness;

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
        border: Option<Border>, 
        shape: Shape, 
        color: Color, 
        transform: Matrix, 
        blend_mode: Pipeline,
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
            Shape::Square => path.add_rectangle(&rect, Winding::Positive),
            Shape::Round(radius) => path.add_rounded_rectangle(
                &rect, 
                &BorderRadii::new(radius), 
                Winding::Positive
            ),

            Shape::RoundSep([
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
        tex: &TextureReference, 
        color: Color, 
        h_flip: bool, 
        v_flip: bool, 
        transform: Matrix, 
        blend_mode: Pipeline,
    ) {
        self.reserve_tex_quad(
            tex, 
            [0.0, 0.0, tex.width as f32, tex.height as f32], 
            color, 
            h_flip, 
            v_flip, 
            transform, 
            blend_mode
        );
    }


    fn draw_slider(
        &mut self,
        quad: [Vector2; 4],
        transform: Matrix,

        mut slider_data: SliderData,
        slider_grids: Vec<GridCell>,
        grid_cells: Vec<u32>,
        line_segments: Vec<LineSegment>
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
            .map(|p| SliderVertex {
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
        quad: [Vector2; 4],
        transform: Matrix,
        flashlight_data: FlashlightData
    ) {
        let Some(mut reserved) = self.reserve_flashlight() 
        else { return };

        let vertices = quad.into_iter()
            .map(|p| FlashlightVertex {
                position: transform.mul_v2(p).into(),
                flashlight_index: reserved.flashlight_index,
            })
            .collect::<Vec<_>>();

        reserved.copy_in(&vertices, flashlight_data);
    }


    fn draw_box_blur(
        &mut self,
        bounds: Bounds,
        size: u32,
    ) {
        let Some(mut reserve) = self.reserve_box_blur() 
        else { return };

        let params = BoxBlurParams::new(
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
        bounds: Bounds,
        sigma: f32,
        _rounds: u32,
    ) {
        let Some(mut reserve) = self.reserve_gaussian_blur() 
        else { return };

        let params = GaussianBlurParams::new(
            bounds.pos.x,
            bounds.pos.y,
            bounds.size.x,
            bounds.size.y,
            sigma,
        );

        reserve.copy_in(params);
    }

    // particle engine stuff
    fn add_emitter(&mut self, emitter: EmitterReference) {
        self.particle_system.add(emitter);
    }

    fn update_emitters(&mut self) {
        self.particle_system.update(&self.device, &self.queue);
    }
}


struct VsyncUtils;
impl VsyncUtils {
    fn map_to_vsync(present_mode: PresentMode) -> Vsync {
        match present_mode {
            PresentMode::AutoVsync => Vsync::AutoVsync,
            PresentMode::AutoNoVsync => Vsync::AutoNoVsync,
            PresentMode::Fifo => Vsync::Fifo,
            PresentMode::FifoRelaxed => Vsync::FifoRelaxed,
            PresentMode::Immediate => Vsync::Immediate,
            PresentMode::Mailbox => Vsync::Mailbox,
        }
    }
    fn map_from_vsync(vsync: Vsync) -> PresentMode {
        match vsync {
            Vsync::AutoVsync => PresentMode::AutoVsync,
            Vsync::AutoNoVsync => PresentMode::AutoNoVsync,
            Vsync::Fifo => PresentMode::Fifo,
            Vsync::FifoRelaxed => PresentMode::FifoRelaxed,
            Vsync::Immediate => PresentMode::Immediate,
            Vsync::Mailbox => PresentMode::Mailbox,
        }
    }
}


fn cast_from_rgba_bytes(bytes: &[u8], format: TextureFormat) -> [u8; 4] {
    // incoming is rgba8
    #[allow(clippy::get_first, reason = "get(0) keeps things lined up here")]
    let r = bytes.get(0).copied().unwrap_or_default();
    let g = bytes.get(1).copied().unwrap_or_default();
    let b = bytes.get(2).copied().unwrap_or_default();
    let a = bytes.get(3).copied().unwrap_or_default();

    match format {
        // pretend this is all it can be for now
        TextureFormat::Bgra8Unorm
        | TextureFormat::Bgra8UnormSrgb => [b, g, r, a],

        // just default to rgba otherwise and cry if its not
        _ => [r, g, b, a]
    }

}

fn cast_to_rgba_bytes(bytes: &[u8], _format: TextureFormat) -> [u8; 4] {
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
    Render(&'a RenderPipeline),
    Compute(&'a ComputePipeline),
}
impl WgpuPipeline<'_> {
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        match self {
            Self::Render(p) => p.get_bind_group_layout(index),
            Self::Compute(p) => p.get_bind_group_layout(index),
        }
    }
}
impl<'a> From<&'a ComputePipeline> for WgpuPipeline<'a> {
    fn from(value: &'a ComputePipeline) -> Self {
        Self::Compute(value)
    }
}
impl<'a> From<&'a RenderPipeline> for WgpuPipeline<'a> {
    fn from(value: &'a RenderPipeline) -> Self {
        Self::Render(value)
    }
}
