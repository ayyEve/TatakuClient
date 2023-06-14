use crate::prelude::*;
use lyon_tessellation::{geom::{Box2D, Point}, path::builder::BorderRadii};
use wgpu::{BufferBinding, util::DeviceExt, TextureViewDimension, ImageCopyBuffer, Extent3d, TextureViewDescriptor};

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
    
    render_pipeline: wgpu::RenderPipeline,
    
    recorded_buffers: Vec<RenderBuffer>,
    queued_buffers: Vec<RenderBuffer>,
    recording_buffer: Option<RenderBuffer>,
    
    //The in-progress CPU side buffers that get uploaded to the GPU upon a call to dump()
    cpu_vtx: Vec<Vertex>,
    cpu_idx: Vec<u32>,
    cpu_scissor: Vec<[f32;4]>,


    // texture_bind_group: wgpu::BindGroup,
    projection_matrix: Matrix,
    projection_matrix_buffer: wgpu::Buffer,
    projection_matrix_bind_group: wgpu::BindGroup,
    // texture_bind_group_layout: wgpu::BindGroupLayout,

    scissor_buffer_layout: wgpu::BindGroupLayout,

    #[allow(unused)]
    sampler: wgpu::Sampler,

    atlas: Atlas,
    render_target_atlas: Atlas,

    atlas_texture: WgpuTexture,

    screenshot_pending: Option<Box<dyn FnOnce((Vec<u8>, u32, u32))+Send+Sync+'static>>
}
impl GraphicsState {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &winit::window::Window, settings: &Settings, size: [u32;2]) -> Self {
        let window_size = settings.window_size;
        let window_size = Vector2::new(window_size[0], window_size[1]);

        // create a wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            // backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(window).unwrap() };

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // create device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER | wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, // | wgpu::Features::BUFFER_BINDING_ARRAY | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ).await.unwrap();
        

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
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/shader.wgsl").into()),
        });
        
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
                    // count: std::num::NonZeroU32::new(layers),
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


        let scissor_buffer_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scissor buffer group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false, 
                        min_binding_size: std::num::NonZeroU64::new(Self::QUAD_PER_BUF * std::mem::size_of::<[f32; 4]>() as u64)
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &projection_matrix_bind_group_layout,
                &texture_bind_group_layout, 
                &scissor_buffer_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, //Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToBorder,
            address_mode_v: wgpu::AddressMode::ClampToBorder,
            address_mode_w: wgpu::AddressMode::ClampToBorder,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }); 

        let atlas_size = device.limits().max_texture_dimension_2d.min(8192);
        let atlas_texture = Self::create_texture(&device, &texture_bind_group_layout, &sampler, atlas_size, atlas_size, config.format);

        let atlas = Atlas::new(atlas_size, atlas_size, LAYER_COUNT);
        let render_target_atlas = Atlas::new(atlas_size, atlas_size, RENDER_TARGET_LAYERS);

        let mut s = Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            render_pipeline,
            sampler,
            atlas,
            render_target_atlas,
            atlas_texture,

            recorded_buffers: Vec::with_capacity(3),
            queued_buffers: Vec::with_capacity(3),
            recording_buffer: None,
            cpu_vtx: vec![Vertex::default(); Self::VTX_PER_BUF as usize],
            cpu_idx: vec![0; Self::IDX_PER_BUF as usize],
            cpu_scissor: vec![[0.0; 4]; Self::QUAD_PER_BUF as usize],


            // texture_bind_group: atlas_texture.bind_group,
            projection_matrix,
            projection_matrix_buffer,
            projection_matrix_bind_group,
            scissor_buffer_layout,
            // texture_bind_group_layout
            screenshot_pending: None
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
        
        // // don't draw if our draw surface has no area
        if output.texture.width() == 0 || output.texture.height() == 0 { return Ok(()) }

        let renderable = RenderableSurface {
            texture: &view,
            clear_color: GFX_CLEAR_COLOR,
        };

        self.render(&renderable)?;
        // self.check_screenshot(&output);

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

            let renderable = RenderableSurface {
                texture: &view,
                clear_color: GFX_CLEAR_COLOR,
            };
            self.render(&renderable)?;

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

            render_pass.set_pipeline(&self.render_pipeline); 
            render_pass.set_bind_group(0, &self.projection_matrix_bind_group, &[]);
            render_pass.set_bind_group(1, &self.atlas_texture.bind_group, &[]);

            for recorded_buffer in self.recorded_buffers.iter() {
                // bind scissors
                render_pass.set_bind_group(2, &recorded_buffer.scissor_buffer_bind_group, &[]);

                render_pass.set_vertex_buffer(0, recorded_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(recorded_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..recorded_buffer.used_indices as u32, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        // output.present();

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


    pub fn create_render_target(&mut self, w:u32, h: u32, clear_color: Color, do_render: impl FnOnce(&mut GraphicsState, Matrix)) -> Option<RenderTarget> {
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
                // Most images are stored using sRGB so we need to reflect that here.
                format: self.config.format,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("render_target_temp_tex"),
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
            label: Some("render_target_temp_tex_view"),
            dimension: Some(TextureViewDimension::D2),
            base_array_layer: 0,

            ..Default::default()
        });

        // create renderable surface
        let renderable = RenderableSurface {
            texture: &view,
            clear_color: target.clear_color,
        };

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
        self.queue.submit(Some(encoder.finish()));

        // remove temp texture
        texture.destroy();

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

        let view_list = textures.iter().map(|a|&a.1).collect::<Vec<_>>();
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

        WgpuTexture {
            textures: Arc::new(textures),
            bind_group
        }
    }


    pub fn load_texture_path(&mut self, file_path: impl AsRef<Path>) -> TatakuResult<TextureReference> {
        let bytes = std::fs::read(file_path.as_ref())?;
        self.load_texture_bytes(bytes)
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
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let data = data.chunks_exact(4).map(|b|cast_from_rgba_bytes(b, self.config.format)).flatten().collect::<Vec<_>>();

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.atlas_texture.textures.get(info.layer as usize).unwrap().0,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: info.x,
                    y: info.y,
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
        if tex.layer >= LAYER_COUNT {
            tex.layer -= LAYER_COUNT;
            self.render_target_atlas.remove_entry(tex)
        } else {
            self.atlas.remove_entry(tex)
        }
    }
    
    pub fn screenshot(&mut self, callback: impl FnOnce((Vec<u8>, u32, u32))+Send+Sync+'static) {
        self.screenshot_pending = Some(Box::new(callback));
    }
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

        let bpr = w*4;
        let tex_buffer = ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout { 
                offset: 0, 
                bytes_per_row: Some(bpr + bpr % 256), 
                rows_per_image: Some(h)
            },
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
    const QUAD_PER_BUF:u64 = 5000;
    const VTX_PER_BUF:u64 = Self::QUAD_PER_BUF * 4;
    const IDX_PER_BUF:u64 = Self::QUAD_PER_BUF * 6;

    fn create_render_buffer(&mut self) {
        let scissor_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scissor Buffer"),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            size: Self::QUAD_PER_BUF * std::mem::size_of::<[f32; 4]>() as u64,
            mapped_at_creation: false,
        });

        let scissor_buffer_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scissor buffer bind group"),
            layout: &self.scissor_buffer_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(scissor_buffer.as_entire_buffer_binding()),
                },
            ],
        });

        self.queued_buffers.push(RenderBuffer {
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
            scissor_buffer,
            scissor_buffer_bind_group,
            used_vertices: 0,
            used_indices: 0,
            // offset by 1 since index 0 should always be [0.0;4], since it means no scissor
            used_scissors: 0,
        })
    }

    pub fn begin(&mut self) {
        // Go through all recorded buffers, and set their used counts to 0, resetting them for the next use
        for i in self.recorded_buffers.iter_mut() {
            i.used_indices = 0;
            i.used_vertices = 0;
            i.used_scissors = 0;
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
                self.queue.write_buffer(&recording_buffer.scissor_buffer, 0, bytemuck::cast_slice(&self.cpu_scissor));
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
    ) -> Option<ReserveData> {
        let mut recording_buffer = self.recording_buffer.as_mut()?;

        if recording_buffer.used_vertices + vtx_count > Self::VTX_PER_BUF 
        || recording_buffer.used_indices + idx_count > Self::IDX_PER_BUF {
            drop(recording_buffer);
            self.dump();

            if self.queued_buffers.is_empty() {
                self.create_render_buffer();
            }

            self.recording_buffer = self.queued_buffers.pop();
            recording_buffer = self.recording_buffer.as_mut()?;
        }

        recording_buffer.used_indices += idx_count;
        recording_buffer.used_vertices += vtx_count;
        let mut scissor_index = 0;

        if let Some(scissor) = scissor {
            // map from [x,y,w,h] to [x1,y1,x2,y2]
            self.cpu_scissor[recording_buffer.used_scissors as usize] = [
                scissor[0], 
                scissor[1],
                scissor[0] + scissor[2],
                scissor[1] + scissor[3]
            ];
            recording_buffer.used_scissors += 1;
            scissor_index = recording_buffer.used_scissors;
        }
        
        Some(ReserveData {
            vtx: &mut self.cpu_vtx[(recording_buffer.used_vertices - vtx_count) as usize .. recording_buffer.used_vertices as usize],
            idx: &mut self.cpu_idx[(recording_buffer.used_indices - idx_count) as usize .. recording_buffer.used_indices as usize],
            idx_offset: recording_buffer.used_vertices - vtx_count,
            scissor_index: scissor_index as u32,
        })
    }


    fn reserve_tex_quad(
        &mut self,
        tex: &TextureReference,
        rect: [f32; 4],
        depth: f32,
        color: Color,
        h_flip: bool,
        v_flip: bool,
        transform: Matrix,
        scissor: Scissor,
    ) {
        let Some(mut reserved) = self.reserve(4, 6, scissor) else { return };
        let depth = Self::map_depth(depth);
        
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
        let scissor_index = reserved.scissor_index;
        reserved.copy_in(&[
            Vertex {
                position: transform.mul_v3(Vector3::new(x, y, depth)).into(),
                tex_coords: tl,
                tex_index,
                color,
                scissor_index,
            },
            Vertex {
                // .position = position + (Gfx.Vector2{ size[0], 0 } * scale),
                position: transform.mul_v3(Vector3::new(x+w, y, depth)).into(),
                tex_coords: tr,
                tex_index,
                color,
                scissor_index,
            },
            Vertex {
                // .position = position + (Gfx.Vector2{ 0, size[1] } * scale),
                position: transform.mul_v3(Vector3::new(x, y+h, depth)).into(),
                tex_coords: bl,
                tex_index,
                color,
                scissor_index,
            },
            Vertex {
                //     .position = position + (size * scale),
                position: transform.mul_v3(Vector3::new(x+w, y+h, depth)).into(),
                tex_coords: br,
                tex_index,
                color,
                scissor_index,
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
        depth: f32,
        color: Color,
        transform: Matrix,
        scissor: Scissor,
    ) {
        let Some(mut reserved) = self.reserve(4, 6, scissor) else { return };
        let depth = Self::map_depth(depth);
        let color = color.into();

        let vertices = quad.into_iter().map(|p|Vertex {
            position: transform.mul_v3(Vector3::new(p.x, p.y, depth)).into(),
            color,
            scissor_index: reserved.scissor_index,
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
    pub fn draw_arc(&mut self, start: f32, end: f32, radius: f32, depth: f32, color: Color, resolution: u32, transform: Matrix, scissor: Scissor) {
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

        self.tessellate_path(path, depth, color, None, transform, scissor);
    }

    pub fn draw_circle(&mut self, radius: f32, depth: f32, color: Color, border: Option<Border>, resolution: u32, transform: Matrix, scissor: Scissor) {
        let n = resolution;

        // border 
        if let Some(border) = border.filter(|b|b.color.a >= 0.0) {
            let radius = radius + border.radius;
            let (x, y, w, h) = (-radius, -radius, 2.0 * radius, 2.0 * radius);
            let (cw, ch) = (0.5 * w, 0.5 * h);
            let (cx, cy) = (x + cw, y + ch);
            let points = (0..n).map(|i| {
                let angle = i as f32 / n as f32 * (PI * 2.0);
                Vector2::new(cx + angle.cos() * cw, cy + angle.sin() * ch)
            });

            self.tessellate_polygon(points, depth - 1.0, border.color, Some(border.radius), transform, scissor);
        }

        // minor optimization
        if color.a <= 0.0 { return }

        // fill
        let (x, y, w, h) = (-radius, -radius, 2.0 * radius, 2.0 * radius);
        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);
        let points = (0..n).map(|i| {
            let angle = i as f32 / n as f32 * (PI * 2.0);
            Vector2::new(cx + angle.cos() * cw, cy + angle.sin() * ch)
        });

        self.tessellate_polygon(points, depth, color, None, transform, scissor);
        
    }

    pub fn draw_line(&mut self, line: [f32; 4], thickness: f32, depth: f32, color: Color, transform: Matrix, scissor: Scissor) {
        let p1 = Vector2::new(line[0], line[1]);
        let p2 = Vector2::new(line[2], line[3]);

        let n = p2 - p1;
        let n = Vector2::new(-n.y, n.x).normalize() * thickness;
    
        let n0 = p1 + n;
        let n1 = p2 + n;
        let n2 = p1 - n;
        let n3 = p2 - n;

        let quad = [ n0, n2, n1, n3 ];
        self.reserve_quad(quad, depth, color, transform, scissor);
    }

    /// rect is [x,y,w,h]
    pub fn draw_rect(&mut self, rect: [f32; 4], depth: f32, border: Option<Border>, shape: Shape, color: Color, transform: Matrix, scissor: Scissor) {
        // for some reason something gets set to infinity on screen resize and panics the tesselator, this prevents that
        if rect.iter().any(|n|!n.is_normal() && *n != 0.0) { return }

        let [x, y, w, h] = rect;
        let rect = Box2D::new(Point::new(x,y), Point::new(x+w, y+h));

        let mut path = lyon_tessellation::path::Path::builder();
        match shape {
            Shape::Square => path.add_rectangle(&rect, lyon_tessellation::path::Winding::Positive),
            Shape::Round(radius, _resolution) => path.add_rounded_rectangle(&rect, &BorderRadii::new(radius), lyon_tessellation::path::Winding::Positive)
        }
        let path = path.build();

        if let Some(border) = border {
            self.tessellate_path(path.clone(), depth, border.color, Some(border.radius), transform, scissor)

            // let points = [
            //     Vector2::new(x, y),
            //     Vector2::new(x+w, y),
            //     Vector2::new(x+w, y+h),
            //     Vector2::new(x, y+h),
            // ].into_iter();
            // self.tesselate_polygon(points, depth-10.0, border.color, Some(border.radius), transform, scissor);
        }

        // minor optimization
        if color.a <= 0.0 { return }

        // self.reserve_rect(rect, depth, color, transform, scissor);
        self.tessellate_path(path, depth, color, None, transform, scissor)
    }

    pub fn draw_tex(&mut self, tex: &TextureReference, depth: f32, color: Color, h_flip: bool, v_flip: bool, transform: Matrix, scissor: Scissor) {
        let rect = [0.0, 0.0, tex.width as f32, tex.height as f32];
        self.reserve_tex_quad(&tex, rect, depth, color, h_flip, v_flip, transform, scissor);
    }


    fn tessellate_polygon(&mut self, mut polygon: impl Iterator<Item=Vector2>, depth: f32, color: Color, border: Option<f32>, transform: Matrix, scissor: Scissor) {
        let depth = Self::map_depth(depth);
        
        let mut path = lyon_tessellation::path::Path::builder();
        path.begin(polygon.next().map(|p|Point::new(p.x, p.y)).unwrap());
        for p in polygon { 
            if !p.x.is_normal() || !p.y.is_normal() { return }
            path.line_to(Point::new(p.x, p.y)); 
        }
        path.end(true);
        let path = path.build();

        self.tessellate_path(path, depth, color, border, transform, scissor)
    }

    fn tessellate_path(&mut self, path: lyon_tessellation::path::Path, depth: f32, color: Color, border: Option<f32>, transform: Matrix, scissor: Scissor) {
        // Create the destination vertex and index buffers.
        let mut buffers: lyon_tessellation::VertexBuffers<Point<f32>, u16> = lyon_tessellation::VertexBuffers::new();
        let depth = Self::map_depth(depth);

        {
            let mut vertex_builder = lyon_tessellation::geometry_builder::simple_builder(&mut buffers);

            let result = if let Some(radius) = border {
                // Create the tessellator.
                let mut tessellator = lyon_tessellation::StrokeTessellator::new();

                // Compute the tessellation.
                tessellator.tessellate_path(
                    &path,
                    &lyon_tessellation::StrokeOptions::default().with_line_width(radius * 2.0),
                    &mut vertex_builder
                )
            } else {
                // Create the tessellator.
                let mut tessellator = lyon_tessellation::FillTessellator::new();

                // Compute the tessellation.
                tessellator.tessellate_path(
                    &path,
                    &lyon_tessellation::FillOptions::default(),
                    &mut vertex_builder
                )
            }; 
            if let Err(e) = result {
                error!("Tesselator error: {e}");
                return;
            }

        }

        let mut reserved = self.reserve(buffers.vertices.len() as u64, buffers.indices.len() as u64, scissor).expect("nope");

        // convert vertices and indices to their proper values
        let mut vertices = buffers.vertices.into_iter().map(|n| Vertex {
                position: [n.x, n.y, depth],
                color: [color.r, color.g, color.b, color.a],
                scissor_index: reserved.scissor_index,
                ..Default::default()
            }.apply_matrix(&transform)
        ).collect::<Vec<_>>();
        
        // insert the vertices and indices into the render buffer
        let mut indices = buffers.indices.into_iter().map(|a|reserved.idx_offset as u32 + a as u32).collect::<Vec<_>>();
        reserved.copy_in(&mut vertices, &mut indices);
    }


    fn map_depth(_d: f32) -> f32 { 0.0 }
}



pub struct WgpuTexture {
    pub textures: Arc<Vec<(wgpu::Texture, wgpu::TextureView)>>,
    // texture: wgpu::Texture,
    // pub texture_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}


pub struct RenderableSurface<'a> {
    texture: &'a wgpu::TextureView,
    clear_color: Color,
}
impl<'a> RenderableSurface<'a> {
    fn get_clear_color(&self) -> wgpu::Color {
        wgpu::Color { 
            r: self.clear_color.r as f64, 
            g: self.clear_color.g as f64, 
            b: self.clear_color.b as f64, 
            a: self.clear_color.a as f64 
        }
    }
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


#[allow(unused)]
#[tokio::test]
async fn test() {
    use winit::{
        event::*,
        event_loop::{ControlFlow, EventLoopBuilder},
        window::WindowBuilder,
        platform::windows::EventLoopBuilderExtWindows
    };

    let settings = Settings::load().await;
    let size = [800, 600];
    
    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = GraphicsState::new(&window, &settings, size).await;

    let tex = state.load_texture_path("C:/Users/Eve/Desktop/Projects/rust/tataku/tataku-client/game/skins/bubbleman/default-4.png").expect("failed to load tex");
    println!("got tex data {tex:?}");


    struct TestThing {
        position: Vector2,
        size: Vector2,
        scale: Vector2,
        rotation: f32,
        tex: TextureReference,
    }
    impl TestThing {
        fn draw(&self, state: &mut GraphicsState) {
            let depth = 0.0;
            let m = 
                Matrix::from_translation(Vector3::new(self.position.x, self.position.y, depth))
                * Matrix::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
                * Matrix::from_angle_z(cgmath::Rad(self.rotation))
            ;
            
            // state.draw_circle(depth, 100.0, Color::GREEN, None, 100, m);

            let angle = PI as f32 / 3.0;
            let p2 = Vector2::from_angle(angle) * 100.0;
            let line = [0.0, 0.0, p2.x, p2.y];
            state.draw_line(line, 5.0, depth, Color::RED, m, None);


            state.draw_tex(&self.tex, depth, Color::WHITE, false, false, m, None);
        }
    }

    let mut things:Vec<TestThing> = Vec::new();
    let mut mouse_pos = Vector2::ZERO;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id } if window_id == window.id() => match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }, ..
            } => *control_flow = ControlFlow::Exit,

            WindowEvent::Resized(physical_size) => state.resize(*physical_size),
            
            // new_inner_size is &&mut so we have to dereference it twice
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.resize(**new_inner_size),
            
            WindowEvent::CursorMoved { position, .. } => {
                mouse_pos = Vector2::new(
                    position.x as f32,
                    position.y as f32,
                )
            }


            WindowEvent::MouseInput { state: ElementState::Pressed, button: winit::event::MouseButton::Left, .. } => {
                println!("mouse: {mouse_pos:?}");
                things.push(TestThing {
                    position: mouse_pos,
                    size: Vector2::ONE * 100.0,
                    scale: Vector2::ONE,
                    rotation: 0.0,
                    tex
                });
            }

            // WindowEvent::KeyboardInput {
            //     input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Left), .. }, ..
            // } => t.position.x -= 100.0,
            // WindowEvent::KeyboardInput {
            //     input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Right), .. }, ..
            // } => t.position.x += 100.0,

            // WindowEvent::KeyboardInput {
            //     input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Up), .. }, ..
            // } => t.position.y -= 100.0,
            // WindowEvent::KeyboardInput {
            //     input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Down), .. }, ..
            // } => t.position.y += 100.0,
            
            _ => {}
        }

        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.begin();
            for i in things.iter_mut() {
                i.draw(&mut state)
            }
            state.end();

            match state.render_current_surface() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                // Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("error: {:?}", e),
            }
        }

        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        
        _ => {}
    });

}
