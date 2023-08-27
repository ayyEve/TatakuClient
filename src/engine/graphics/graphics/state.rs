use crate::prelude::*;
use lyon_tessellation::{geom::{Box2D, Point}, path::builder::BorderRadii};

use wgpu::{ BufferBinding, util::DeviceExt, TextureViewDimension, ImageCopyBuffer, Extent3d, TextureViewDescriptor };

// the sum of these two must not go past 16
const LAYER_COUNT:u32 = 2;
const RENDER_TARGET_LAYERS:u32 = 2;
pub const MAX_DEPTH:f32 = 8192.0 * 8192.0;

/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;

pub type Scissor = Option<[f32; 4]>;

pub struct GraphicsState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    
    pipelines: HashMap<BlendMode, wgpu::RenderPipeline>,
    // blending_pipeline: wgpu::RenderPipeline,
    // no_blending_pipeline: wgpu::RenderPipeline,
    
    recorded_buffers: Vec<RenderBuffer>,
    queued_buffers: Vec<RenderBuffer>,
    recording_buffer: Option<RenderBuffer>,
    
    // The in-progress CPU side buffers that get uploaded to the GPU upon a call to dump()
    cpu_vtx: Vec<Vertex>,
    cpu_idx: Vec<u32>,
    // cpu_scissor: Vec<[f32; 4]>,

    // texture_bind_group: wgpu::BindGroup,
    projection_matrix: Matrix,
    projection_matrix_buffer: wgpu::Buffer,
    projection_matrix_bind_group: wgpu::BindGroup,
    // texture_bind_group_layout: wgpu::BindGroupLayout,

    // scissor_buffer_layout: wgpu::BindGroupLayout,

    atlas: Atlas,
    render_target_atlas: Atlas,
    atlas_texture: WgpuTexture,

    screenshot_pending: Option<Box<dyn FnOnce((Vec<u8>, u32, u32))+Send+Sync+'static>>,

    particle_system: ParticleSystem,
}
impl GraphicsState {

    // Creating some of the wgpu types requires async code
    #[cfg(feature="graphics")]
    pub async fn new(window: &winit::window::Window, settings: &Settings, size: [u32;2]) -> Self {
        let window_size = settings.window_size;
        let window_size = Vector2::new(window_size[0], window_size[1]);

        // create a wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL,
            dx12_shader_compiler: Default::default(),
        });
        
        // create the serface
        let surface = unsafe { instance.create_surface(window).unwrap() };

        // create the adapter
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
                features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                #[cfg(not(feature="texture_arrays"))] 
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();
        
        // no more comments good luck!

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: size[0],
            height: size[1],
            present_mode: wgpu::PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            #[cfg(feature="texture_arrays")] source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/shader_with_tex_array.wgsl").into()),
            #[cfg(not(feature="texture_arrays"))] source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/shader.wgsl").into()),
        });
        
        
        #[cfg(feature="texture_arrays")]
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // count: None,
                    count: std::num::NonZeroU32::new(LAYER_COUNT + RENDER_TARGET_LAYERS),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        #[cfg(not(feature="texture_arrays"))]
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        });

        let proj_matrix_size = std::mem::size_of::<[[f32; 4]; 4]>() as u64;
        let projection_matrix_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture/Sampler bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: std::num::NonZeroU64::new(proj_matrix_size)
                    },
                    count: None,
                },
            ]
        });

        let projection_matrix = Self::create_projection(window_size);
        let projection_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projection Matrix Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&projection_matrix.to_raw()),
        });

        let projection_matrix_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &projection_matrix_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(BufferBinding {
                            buffer: &projection_matrix_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(proj_matrix_size)
                        }),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        );


        // let scissor_buffer_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("scissor buffer group layout"),
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Buffer { 
        //                 ty: wgpu::BufferBindingType::Uniform, 
        //                 has_dynamic_offset: false, 
        //                 min_binding_size: std::num::NonZeroU64::new(Self::QUAD_PER_BUF * std::mem::size_of::<[f32; 4]>() as u64)
        //             },
        //             count: None,
        //         },
        //     ],
        // });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &projection_matrix_bind_group_layout,
                &texture_bind_group_layout, 
                // &scissor_buffer_layout
            ],
            push_constant_ranges: &[],
        });


        let mut pipelines = HashMap::new();

        for (blend_mode, blend_state) in [
            (BlendMode::AlphaBlending, wgpu::BlendState::ALPHA_BLENDING),
            (BlendMode::AlphaOverwrite, wgpu::BlendState::REPLACE),
            (BlendMode::PremultipliedAlpha, wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            (BlendMode::AdditiveBlending, wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
            }),
            (BlendMode::SourceAlphaBlending, wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
            }),
        ] {
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{blend_mode:?} Pipeline")),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[ Vertex::desc() ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(blend_state),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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


        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }); 

        let atlas_size = device.limits().max_texture_dimension_2d.min(8192);
        let atlas_texture = Self::create_texture(&device, &texture_bind_group_layout, &sampler, atlas_size, atlas_size, config.format);

        let atlas = Atlas::new(atlas_size, atlas_size, LAYER_COUNT);
        let render_target_atlas = Atlas::new(atlas_size, atlas_size, RENDER_TARGET_LAYERS);

        let particle_system = ParticleSystem::new(&device);

        let mut s = Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            pipelines,
            atlas,
            render_target_atlas,
            atlas_texture,

            recorded_buffers: Vec::with_capacity(3),
            queued_buffers: Vec::with_capacity(3),
            recording_buffer: None,

            cpu_vtx: vec![Vertex::default(); Self::VTX_PER_BUF as usize],
            cpu_idx: vec![0; Self::IDX_PER_BUF as usize],
            // cpu_scissor: vec![[0.0; 4]; Self::QUAD_PER_BUF as usize],

            // texture_bind_group: atlas_texture.bind_group,
            projection_matrix,
            projection_matrix_buffer,
            projection_matrix_bind_group,
            // scissor_buffer_layout,
            // texture_bind_group_layout
            screenshot_pending: None,
            particle_system,
        };
        s.create_render_buffer();
        s
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let window_size = Vector2::new(new_size.width as f32, new_size.height as f32);
            self.projection_matrix = Self::create_projection(window_size);
            self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&self.projection_matrix.to_raw()));
        }
    }

    pub fn set_vsync(&mut self, enable: bool) {
        if enable {
            self.config.present_mode = wgpu::PresentMode::AutoVsync;
        } else {
            self.config.present_mode = wgpu::PresentMode::AutoNoVsync;
        }

        self.surface.configure(&self.device, &self.config);
    }


    pub fn render_current_surface(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let size = output.texture.size();
        
        // don't draw if our draw surface has no area
        if size.width == 0 || size.height == 0 { return Ok(()) }
        
        self.render(&RenderableSurface::new(&view, GFX_CLEAR_COLOR, Vector2::new(size.width as f32, size.height as f32)))?;

        let width = output.texture.width();
        let height = output.texture.height();

        output.present();

        if let Some(screenshot) = std::mem::take(&mut self.screenshot_pending) {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor { 
                label: Some("Screenshot Texture"), 
                size: Extent3d { 
                    width, 
                    height, 
                    depth_or_array_layers: 1
                }, 
                mip_level_count: 1, 
                sample_count: 1, 
                dimension: wgpu::TextureDimension::D2, 
                format: wgpu::TextureFormat::Bgra8UnormSrgb, 
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, 
                view_formats: &[]
            });
            let view = texture.create_view(&TextureViewDescriptor {
                label: Some("Screenshot Texture View"),
                dimension: Some(TextureViewDimension::D2),
                base_array_layer: 0,

                ..Default::default()
            });

            self.render(&RenderableSurface::new(&view, GFX_CLEAR_COLOR, Vector2::new(width as f32, height as f32)))?;

            #[cfg(feature="graphics")]
            self.finish_screenshot(texture, screenshot);
        }


        Ok(())
    }

    pub fn render(&self, renderable: &RenderableSurface) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &renderable.texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(renderable.get_clear_color()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let mut current_blend_mode = BlendMode::None;
            let mut current_scissor: Scissor = None;

            for recorded_buffer in self.recorded_buffers.iter() {
                if recorded_buffer.blend_mode != current_blend_mode {
                    current_blend_mode = recorded_buffer.blend_mode;
                    let Some(pipeline) = self.pipelines.get(&recorded_buffer.blend_mode) else { 
                        error!("Pipeline not created for blend mode {current_blend_mode:?}");
                        current_blend_mode = BlendMode::None;
                        continue 
                    };

                    render_pass.set_pipeline(&pipeline);
                    render_pass.set_bind_group(0, &self.projection_matrix_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.atlas_texture.bind_group, &[]);
                }

                if recorded_buffer.scissor.unwrap() != current_scissor {
                    current_scissor = recorded_buffer.scissor.unwrap();
                    let [x, y, w, h] = current_scissor.unwrap_or_else(||[0.0, 0.0, renderable.size.x, renderable.size.y]);

                    render_pass.set_scissor_rect(
                        x.clamp(0.0, renderable.size.x) as u32, 
                        y.clamp(0.0, renderable.size.y) as u32, 
                        w.clamp(0.0, renderable.size.x - x) as u32, 
                        h.clamp(0.0, renderable.size.y - y) as u32
                    );
                }

                render_pass.set_vertex_buffer(0, recorded_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(recorded_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..recorded_buffer.used_indices as u32, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit([encoder.finish()]);

        Ok(())
    }


    fn create_projection(draw_size: Vector2) -> Matrix {
        let sx = (2.0 / draw_size.x) as f32;
        let sy = (-2.0 / draw_size.y) as f32;

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


    pub fn create_render_target(&mut self, w:u32, h:u32, clear_color: Color, do_render: impl FnOnce(&mut GraphicsState, Matrix)) -> Option<RenderTarget> {
        // find space in the render target atlas
        let mut atlased = self.render_target_atlas.try_insert(w, h)?;

        // offset the texture layer so it accesses the render target atlas
        atlased.layer += LAYER_COUNT;

        // create a projection and render target
        let projection = Self::create_projection(Vector2::new(w as f32, h as f32));
        let target = RenderTarget::new_main_thread(w, h, atlased, projection, clear_color);

        // queue rendering the data to it
        self.update_render_target(target.clone(), do_render);

        // return the new render target
        Some(target)
    }
    pub fn update_render_target(&mut self, target: RenderTarget, do_render: impl FnOnce(&mut GraphicsState, Matrix)) {
        // get the texture this target was written to
        let textures = self.atlas_texture.textures.clone();
        let Some((atlas_tex, _)) = textures.get(target.texture.layer as usize) else { return };

        // write the projection matrix
        self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&target.projection.to_raw()));
        self.queue.submit([].into_iter());

        let width = target.width;
        let height = target.height;

        // create a temporary texture to render to this target to
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
                format: self.config.format,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("render_target_temp_tex"),
                view_formats: &[],
            }
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("render_target_temp_tex_view"),
            dimension: Some(TextureViewDimension::D2),
            base_array_layer: 0,

            ..Default::default()
        });

        // create renderable surface
        let renderable = RenderableSurface::new(&view, target.clear_color, Vector2::new(width as f32, height as f32));

        // clear buffers
        self.begin();

        // fill buffers
        let transform = Matrix::identity();
        do_render(self, transform);

        // complete buffers
        self.end();

        // perform render
        let _ = self.render(&renderable);


        // copy render to atlas
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_target copy encoder") });

        let mut dest = atlas_tex.as_image_copy();
        dest.origin.x = target.texture.x;
        dest.origin.y = target.texture.y;
        
        encoder.copy_texture_to_texture(texture.as_image_copy(), dest, Extent3d { width, height, depth_or_array_layers: 1 });
        self.queue.submit([encoder.finish()]);

        // remove temp texture
        self.queue.on_submitted_work_done(move || texture.destroy());

        // reapply the window projection matrix
        self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&self.projection_matrix.to_raw()));

    }
}

// texture stuff
impl GraphicsState {
    fn create_texture(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, width:u32, height:u32, format: wgpu::TextureFormat) -> WgpuTexture {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let textures = (0..(LAYER_COUNT+RENDER_TARGET_LAYERS)).map(|_| {
            let texture = device.create_texture(
                &wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    // Most images are stored using sRGB so we need to reflect that here.
                    format, //wgpu::TextureFormat::Rgba8UnormSrgb,
                    // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    label: Some("atlas_texture"),
                    // This is the same as with the SurfaceConfig. It
                    // specifies what texture formats can be used to
                    // create TextureViews for this texture. The base
                    // texture format (Rgba8UnormSrgb in this case) is
                    // always supported. Note that using a different
                    // texture format is not supported on the WebGL2
                    // backend.
                    view_formats: &[],
                }
            );
            
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("atlas_texture_view"),
                dimension: Some(TextureViewDimension::D2),
                base_array_layer: 0,

                ..Default::default()
            });
            (texture, view)
        })
        // .chain((0..RENDER_TARGET_LAYERS).map(|_| {
        //     let texture = device.create_texture(
        //         &wgpu::TextureDescriptor {
        //             size: texture_size,
        //             mip_level_count: 1,
        //             sample_count: 1,
        //             dimension: wgpu::TextureDimension::D2,
        //             // Most images are stored using sRGB so we need to reflect that here.
        //             format,
        //             // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
        //             // COPY_DST means that we want to copy data to this texture
        //             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        //             label: Some("render_target_texture_atlas"),
        //             // This is the same as with the SurfaceConfig. It
        //             // specifies what texture formats can be used to
        //             // create TextureViews for this texture. The base
        //             // texture format (Rgba8UnormSrgb in this case) is
        //             // always supported. Note that using a different
        //             // texture format is not supported on the WebGL2
        //             // backend.
        //             view_formats: &[],
        //         }
        //     );
        //     let view = texture.create_view(&wgpu::TextureViewDescriptor {
        //         label: Some("pain and suffering"),
        //         dimension: Some(TextureViewDimension::D2),
        //         base_array_layer: 0,
        //         ..Default::default()
        //     });
        //     (texture, view)
        // }))
        .collect::<Vec<_>>();


        #[cfg(feature="texture_arrays")]
        let view_list = textures.iter().map(|a|&a.1).collect::<Vec<_>>();
        #[cfg(feature="texture_arrays")]
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture array bind group"),
            layout: &layout,
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
        });

        #[cfg(not(feature="texture_arrays"))]
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture array bind group"),
            layout: &layout,
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

    pub fn load_texture_bytes(&mut self, data: impl AsRef<[u8]>) -> TatakuResult<TextureReference> {
        let diffuse_image = image::load_from_memory(data.as_ref())?;
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let (width, height) = diffuse_image.dimensions();

        self.load_texture_rgba(&diffuse_rgba.to_vec(), width, height)
    }

    pub fn load_texture_rgba(&mut self, data: &Vec<u8>, width: u32, height: u32) -> TatakuResult<TextureReference> {
        let Some(info) = self.atlas.try_insert(width, height) else { return Err(TatakuError::String("no space in atlas".to_owned())); };
        if info.is_empty() { return Ok(info) }

        // let padding_bytes = (0..ATLAS_PADDING).map(|_|[0u8;4]).flatten().collect::<Vec<u8>>();

        let data = data
        // cast to bgra
        .chunks_exact(4).map(|b|cast_from_rgba_bytes(b, self.config.format)).flatten().collect::<Vec<_>>()
        // // add padding bytes to both left and right side
        // .chunks_exact(4 * width as usize).map(|b|[&padding_bytes[..], b, &padding_bytes[..]]).flatten()
        // // collect into Vec<u8>
        // .flatten()
        // .map(|b|*b)
        // .collect::<Vec<_>>()
        ;
    
    
        // let width = width + ATLAS_PADDING * 2;
        // let height = height + ATLAS_PADDING * 2;

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };


        // let vertical_padding = vec![0u8; (width * 4 * ATLAS_PADDING) as usize];
        // let mut data2 = vertical_padding.clone();
        // data2.extend(data.into_iter());
        // data2.extend(vertical_padding.into_iter());
        

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.atlas_texture.textures.get(info.layer as usize).unwrap().0,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: info.x, // x: info.x - ATLAS_PADDING,
                    y: info.y, // y: info.y - ATLAS_PADDING,
                    z: 0
                },
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &data,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        Ok(info)
    }

    pub fn free_tex(&mut self, mut tex: TextureReference) {
        if tex.is_empty() { return }

        // write empty data to where the texture was
        // this should remove the weird border when the atlas space is reused
        let width = tex.width + ATLAS_PADDING * 2;
        let height = tex.height + ATLAS_PADDING * 2;
        // empty pixels
        let data = vec![0u8; (width * height * 4) as usize];

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.atlas_texture.textures.get(tex.layer as usize).unwrap().0,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: tex.x - ATLAS_PADDING,
                    y: tex.y - ATLAS_PADDING,
                    z: 0
                },
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &data,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            }
        );

        // remove from texture atlas
        if tex.layer >= LAYER_COUNT {
            tex.layer -= LAYER_COUNT;
            self.render_target_atlas.remove_entry(tex);
        } else {
            self.atlas.remove_entry(tex);
        }
    }
    
    #[cfg(feature="graphics")]
    pub fn screenshot(&mut self, callback: impl FnOnce((Vec<u8>, u32, u32))+Send+Sync+'static) {
        self.screenshot_pending = Some(Box::new(callback));
    }
    
    #[cfg(feature="graphics")]
    fn finish_screenshot(&mut self, texture: wgpu::Texture, callback: Box<dyn FnOnce((Vec<u8>, u32, u32)) + Send + Sync>) {
        let (w, h) = (texture.width(), texture.height());
        let format = texture.format();

        let size = (w * h * 4) as u64;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Buffer"),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            size,
            mapped_at_creation: false,
        });


        let tex_buffer = ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout { 
                offset: 0, 
                bytes_per_row: Some(align(w*4)), 
                rows_per_image: Some(h)
            }
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("screenshot encoder") });
        encoder.copy_texture_to_buffer(texture.as_image_copy(), tex_buffer, Extent3d { width: w, height: h, depth_or_array_layers: 1 });
        self.queue.submit(Some(encoder.finish()));
        let queue = self.queue.clone();

        tokio::spawn(async move {
            let slice = buffer.slice(..);

            let (s, r) = tokio::sync::oneshot::channel();
            slice.map_async(wgpu::MapMode::Read, move |_result| s.send(()).unwrap());
            queue.submit(None);
            
            r.await.unwrap();
            let data = slice.get_mapped_range().chunks_exact(4).map(|b|cast_to_rgba_bytes(b, format)).flatten().collect();
            
            callback((data, w, h));
        });
    }
}


// render code
impl GraphicsState {
    const QUAD_PER_BUF:u64 = 3000;
    const VTX_PER_BUF:u64 = Self::QUAD_PER_BUF * 4;
    const IDX_PER_BUF:u64 = Self::QUAD_PER_BUF * 6;

    fn create_render_buffer(&mut self) {
        self.queued_buffers.push(RenderBuffer {
            blend_mode: BlendMode::None,
            scissor: None,
            vertex_buffer: self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: Self::VTX_PER_BUF * std::mem::size_of::<Vertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size: Self::IDX_PER_BUF * std::mem::size_of::<u32>() as u64,
                mapped_at_creation: false,
            }),
            used_vertices: 0,
            used_indices: 0,
        })
    }

    pub fn begin(&mut self) {
        // Go through all recorded buffers, and set their used counts to 0, resetting them for the next use
        for i in self.recorded_buffers.iter_mut() {
            i.used_indices = 0;
            i.used_vertices = 0;
            i.blend_mode = BlendMode::None;
            i.scissor = None;
        }
        
        // Move all recorded buffers into the queued buffers list
        self.queued_buffers.append(&mut self.recorded_buffers);
        self.recording_buffer = Some(self.queued_buffers.pop().unwrap());
        // self.started = true;
    }

    pub fn end(&mut self) {
        self.dump();
    }

    fn dump(&mut self) {
        if let Some(recording_buffer) = std::mem::take(&mut self.recording_buffer) {
            if recording_buffer.used_indices != 0 {
                self.queue.write_buffer(&recording_buffer.vertex_buffer, 0, bytemuck::cast_slice(&self.cpu_vtx));
                self.queue.write_buffer(&recording_buffer.index_buffer, 0, bytemuck::cast_slice(&self.cpu_idx));
                self.recorded_buffers.push(recording_buffer);
            } else {
                self.queued_buffers.push(recording_buffer);
            }
        }
    }

    /// returns reserve data and scissor index if applicable
    fn reserve(
        &mut self,
        vtx_count: u64,
        idx_count: u64,
        scissor: Scissor,
        blend_mode: BlendMode,
    ) -> Option<ReserveData> {
        let mut recording_buffer = self.recording_buffer.as_mut()?;
        let blend_mode_check = recording_buffer.blend_mode == blend_mode || recording_buffer.blend_mode == BlendMode::None;
        let scissor_check = recording_buffer.scissor == Some(scissor) || recording_buffer.scissor == None;

        // if !blend_mode_check { println!("blend mode changed from {:?} to {blend_mode:?}", recording_buffer.blend_mode) }

        if !blend_mode_check 
        || !scissor_check
        || recording_buffer.used_vertices + vtx_count > Self::VTX_PER_BUF 
        || recording_buffer.used_indices + idx_count > Self::IDX_PER_BUF {
            self.dump();

            if self.queued_buffers.is_empty() {
                self.create_render_buffer();
            }

            self.recording_buffer = self.queued_buffers.pop();
            recording_buffer = self.recording_buffer.as_mut()?;
            recording_buffer.blend_mode = blend_mode;
            recording_buffer.scissor = Some(scissor);
        }
        if recording_buffer.blend_mode == BlendMode::None {
            recording_buffer.blend_mode = blend_mode;
        }
        if recording_buffer.scissor.is_none() {
            recording_buffer.scissor = Some(scissor);
        }

        recording_buffer.used_indices += idx_count;
        recording_buffer.used_vertices += vtx_count;
        
        Some(ReserveData {
            vtx: &mut self.cpu_vtx[(recording_buffer.used_vertices - vtx_count) as usize .. recording_buffer.used_vertices as usize],
            idx: &mut self.cpu_idx[(recording_buffer.used_indices - idx_count) as usize .. recording_buffer.used_indices as usize],
            idx_offset: recording_buffer.used_vertices - vtx_count,
        })
    }


    fn reserve_tex_quad(
        &mut self,
        tex: &TextureReference,
        rect: [f32; 4],
        color: Color,
        h_flip: bool,
        v_flip: bool,
        transform: Matrix,
        scissor: Scissor,
        blend_mode: BlendMode,
    ) {
        let Some(mut reserved) = self.reserve(4, 6, scissor, blend_mode) else { return };
        
        let [x, y, w, h] = rect;
        let color = color.into();

        let mut tl = tex.uvs.tl.into();
        let mut tr = tex.uvs.tr.into();
        let mut bl = tex.uvs.bl.into();
        let mut br = tex.uvs.br.into();

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
        reserved.copy_in(&[
            Vertex {
                position: transform.mul_v2(Vector2::new(x, y)).into(),
                tex_coords: tl,
                tex_index,
                color,
            },
            Vertex {
                // .position = position + (Gfx.Vector2{ size[0], 0 } * scale),
                position: transform.mul_v2(Vector2::new(x+w, y)).into(),
                tex_coords: tr,
                tex_index,
                color,
            },
            Vertex {
                // .position = position + (Gfx.Vector2{ 0, size[1] } * scale),
                position: transform.mul_v2(Vector2::new(x, y+h)).into(),
                tex_coords: bl,
                tex_index,
                color,
            },
            Vertex {
                //     .position = position + (size * scale),
                position: transform.mul_v2(Vector2::new(x+w, y+h)).into(),
                tex_coords: br,
                tex_index,
                color,
            }
        ], &[
            0 + offset,
            2 + offset,
            1 + offset,
            1 + offset,
            2 + offset,
            3 + offset,
        ]);
    }

    // quad is tl,tr, bl,br
    fn reserve_quad(
        &mut self,
        quad: [Vector2; 4],
        color: Color,
        transform: Matrix,
        scissor: Scissor,
        blend_mode: BlendMode,
    ) {
        let Some(mut reserved) = self.reserve(4, 6, scissor, blend_mode) else { return };
        let color = color.into();

        let vertices = quad.into_iter().map(|p: Vector2|Vertex {
            position: transform.mul_v2(p).into(),
            color,
            ..Default::default()
        }).collect::<Vec<_>>();
        
        let offset = reserved.idx_offset as u32;
        reserved.copy_in(&vertices, &[
            0 + offset,
            2 + offset,
            1 + offset,

            1 + offset,
            2 + offset,
            3 + offset,
        ]);
    }

}


// draw helpers
impl GraphicsState {

    /// draw an arc with the center at 0,0
    pub fn draw_arc(&mut self, start: f32, end: f32, radius: f32, color: Color, resolution: u32, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        let n = resolution;
        
        // minor optimization
        if color.a <= 0.0 { return }

        let (x, y, w, h) = (-radius, -radius, 2.0 * radius, 2.0 * radius);
        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);

        let mut path = lyon_tessellation::path::Path::builder();
        for i in 0..=n {
            let angle = f32::lerp(start, end, i as f32 / n as f32);
            let p = Point::new(cx + angle.cos() * cw, cy + angle.sin() * ch);
            if i == 0 { 
                path.begin(p); 
            } else {
                path.line_to(p);
            }
        }
        path.end(false);
        let path = path.build();

        self.tessellate_path(&path, color, None, transform, scissor, blend_mode);
    }

    pub fn draw_circle(&mut self, radius: f32, color: Color, border: Option<Border>, resolution: u32, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        let n = resolution;

        let (x, y, w, h) = (-radius, -radius, 2.0 * radius, 2.0 * radius);
        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);
        let points = (0..n).map(|i| {
            let angle = i as f32 / n as f32 * (PI * 2.0);
            Vector2::new(cx + angle.cos() * cw, cy + angle.sin() * ch)
        }).collect::<Vec<_>>();

        // fill
        if color.a > 0.0 { 
            self.tessellate_polygon(&points, color, None, transform, scissor, blend_mode);
        }
        
        // border 
        if let Some(border) = border.filter(|b|b.color.a > 0.0) {
            // let radius = radius + border.radius;
            // let (x, y, w, h) = (-radius, -radius, 2.0 * radius, 2.0 * radius);
            // let (cw, ch) = (0.5 * w, 0.5 * h);
            // let (cx, cy) = (x + cw, y + ch);
            // let points = (0..n).map(|i| {
            //     let angle = i as f32 / n as f32 * (PI * 2.0);
            //     Vector2::new(cx + angle.cos() * cw, cy + angle.sin() * ch)
            // });

            self.tessellate_polygon(&points, border.color, Some(border.radius), transform, scissor, blend_mode);
        }

    }

    pub fn draw_line(&mut self, line: [f32; 4], thickness: f32, color: Color, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        let p1 = Vector2::new(line[0], line[1]);
        let p2 = Vector2::new(line[2], line[3]);

        let n = p2 - p1;
        let n = Vector2::new(-n.y, n.x).normalize() * thickness;
    
        let n0 = p1 + n;
        let n1 = p2 + n;
        let n2 = p1 - n;
        let n3 = p2 - n;

        let quad = [ n0, n2, n1, n3 ];
        self.reserve_quad(quad, color, transform, scissor, blend_mode);
    }

    /// rect is [x,y,w,h]
    pub fn draw_rect(&mut self, rect: [f32; 4], border: Option<Border>, shape: Shape, color: Color, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        // for some reason something gets set to infinity on screen resize and panics the tesselator, this prevents that
        if rect.iter().any(|n|!n.is_normal() && *n != 0.0) { return }

        let [x, y, w, h] = rect;
        let rect = Box2D::new(Point::new(x, y), Point::new(x+w, y+h));

        let mut path = lyon_tessellation::path::Path::builder();
        match shape {
            Shape::Square => path.add_rectangle(&rect, lyon_tessellation::path::Winding::Positive),
            Shape::Round(radius) => path.add_rounded_rectangle(&rect, &BorderRadii::new(radius), lyon_tessellation::path::Winding::Positive)
        }
        let path = path.build();

        // fill
        if color.a > 0.0 { 
            self.tessellate_path(&path, color, None, transform, scissor, blend_mode)
        }

        // border
        if let Some(border) = border.filter(|b|b.color.a > 0.0) {
            self.tessellate_path(&path, border.color, Some(border.radius), transform, scissor, blend_mode)
        }
    }

    pub fn draw_tex(&mut self, tex: &TextureReference, color: Color, h_flip: bool, v_flip: bool, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        let rect = [0.0, 0.0, tex.width as f32, tex.height as f32];
        self.reserve_tex_quad(&tex, rect, color, h_flip, v_flip, transform, scissor, blend_mode);
    }


    fn tessellate_polygon(&mut self, polygon: &Vec<Vector2>, color: Color, border: Option<f32>, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        let mut polygon = polygon.iter();
        let mut path = lyon_tessellation::path::Path::builder();
        path.begin(polygon.next().map(|p|Point::new(p.x, p.y)).unwrap());
        for p in polygon { 
            if !p.x.is_normal() || !p.y.is_normal() { return }
            path.line_to(Point::new(p.x, p.y)); 
        }
        path.end(true);
        let path = path.build();

        self.tessellate_path(&path, color, border, transform, scissor, blend_mode)
    }

    fn tessellate_path(&mut self, path: &lyon_tessellation::path::Path, color: Color, border: Option<f32>, transform: Matrix, scissor: Scissor, blend_mode: BlendMode) {
        // Create the destination vertex and index buffers.
        let mut buffers: lyon_tessellation::VertexBuffers<Point<f32>, u16> = lyon_tessellation::VertexBuffers::new();

        {
            let mut vertex_builder = lyon_tessellation::geometry_builder::simple_builder(&mut buffers);

            let result = if let Some(radius) = border {
                // Create the tessellator.
                let mut tessellator = lyon_tessellation::StrokeTessellator::new();

                // Compute the tessellation.
                tessellator.tessellate_path(
                    path,
                    &lyon_tessellation::StrokeOptions::default().with_line_width(radius * 2.0),
                    &mut vertex_builder
                )
            } else {
                // Create the tessellator.
                let mut tessellator = lyon_tessellation::FillTessellator::new();

                // Compute the tessellation.
                tessellator.tessellate_path(
                    path,
                    &lyon_tessellation::FillOptions::default(),
                    &mut vertex_builder
                )
            }; 
            if let Err(e) = result {
                error!("Tesselator error: {e}");
                return;
            }

        }

        let mut reserved = self.reserve(buffers.vertices.len() as u64, buffers.indices.len() as u64, scissor, blend_mode).expect("nope");

        // convert vertices and indices to their proper values
        let mut vertices = buffers.vertices.into_iter().map(|n| Vertex {
                position: [n.x, n.y],
                color: [color.r, color.g, color.b, color.a],
                // scissor_index: reserved.scissor_index,
                ..Default::default()
            }.apply_matrix(&transform)
        ).collect::<Vec<_>>();
        
        // insert the vertices and indices into the render buffer
        let mut indices = buffers.indices.into_iter().map(|a|reserved.idx_offset as u32 + a as u32).collect::<Vec<_>>();
        reserved.copy_in(&mut vertices, &mut indices);
    }
}

// particle stuff 
impl GraphicsState {
    pub fn add_emitter(&mut self, emitter: EmitterRef) {
        self.particle_system.add(emitter);
    }

    pub fn update_emitters(&mut self) {
        self.particle_system.update(&self.device, &self.queue);
    }
}


pub struct WgpuTexture {
    pub textures: Arc<Vec<(wgpu::Texture, wgpu::TextureView)>>,
    pub bind_group: wgpu::BindGroup,
}


pub struct RenderableSurface<'a> {
    texture: &'a wgpu::TextureView,
    size: Vector2,
    clear_color: Color,
}
impl<'a> RenderableSurface<'a> {
    fn new(texture: &'a wgpu::TextureView, clear_color: Color, size: Vector2) -> Self {
        Self {
            texture,
            size,
            clear_color
        }
    }
    fn get_clear_color(&self) -> wgpu::Color {
        wgpu::Color { 
            r: self.clear_color.r as f64, 
            g: self.clear_color.g as f64, 
            b: self.clear_color.b as f64, 
            a: self.clear_color.a as f64 
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, Dropdown, Eq, PartialEq)]
pub enum PerformanceMode {
    PowerSaver,
    HighPerformance,
}


fn cast_from_rgba_bytes(bytes: &[u8], format: wgpu::TextureFormat) -> [u8; 4] {
    // incoming is rgba8
    let r = bytes.get(0).cloned().unwrap_or_default();
    let g = bytes.get(1).cloned().unwrap_or_default();
    let b = bytes.get(2).cloned().unwrap_or_default();
    let a = bytes.get(3).cloned().unwrap_or_default();

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
    let b = bytes.get(1).cloned().unwrap_or_default();
    let g = bytes.get(1).cloned().unwrap_or_default();
    let r = bytes.get(2).cloned().unwrap_or_default();
    let a = bytes.get(3).cloned().unwrap_or_default();

    [r,g,b,a]

    // match format {
    //     // pretend this is all it can be for now
    //     wgpu::TextureFormat::Bgra8Unorm
    //     | wgpu::TextureFormat::Bgra8UnormSrgb => [b, g, r, a],
    //     wgpu::TextureFormat::Rgba8Unorm

    //     // just default to rgba otherwise and cry if its not
    //     _ => [r, g, b, a]
    // }

}

/// pad `num` to align with `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`
#[cfg(feature="graphics")]
fn align(num: u32) -> u32 {
    let m = num % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    if m == 0 {
        num
    } else {
        num + (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - m)
    }
}
