use crate::prelude::*;
use super::shaders::*;

use tataku_engine::prelude::*;
use lyon_tessellation::{ 
    geom::{ Box2D, Point }, 
    path::builder::BorderRadii 
};

use wgpu::{ 
    Extent3d, 
    BufferBinding, 
    ImageCopyBuffer, 
    util::DeviceExt, 
    TextureViewDescriptor,
    TextureViewDimension, 
};

use winit::raw_window_handle::{ HasWindowHandle, HasDisplayHandle };

// must not go past 16
const LAYER_COUNT:u32 = 4;
const MAX_DEPTH:f32 = 8192.0 * 8192.0;

/// background color
const GFX_CLEAR_COLOR:Color = Color::BLACK;

macro_rules! get_render_buffer {
    ($self: ident, $t: ident) => {{
        let b = $self.current_render_buffer.as_mut().expect("last drawn type not set");
        if let RenderBufferQueueType::$t(b2) = &mut **b {b2} else { panic!("wrong buffer type") }
    }}
}

pub struct WgpuEngine<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,

    pipelines: HashMap<BlendMode, wgpu::RenderPipeline>,

    buffer_queues: HashMap<LastDrawn, Box<RenderBufferQueueType>>,
    completed_buffers: Vec<RenderBufferType>,
    current_render_buffer: Option<Box<RenderBufferQueueType>>,

    projection_matrix: Matrix,
    projection_matrix_buffer: wgpu::Buffer,
    projection_matrix_bind_group: wgpu::BindGroup,

    atlas: Atlas,
    atlas_texture: WgpuTexture,

    screenshot_pending: Option<Box<dyn FnOnce((Vec<u8>, [u32; 2]))+Send+Sync>>,

    particle_system: ParticleSystem,

    scissors: ScissorManager,

    present_modes: Vec<Vsync>
}
impl<'window> WgpuEngine<'window> {

    // Creating some of the wgpu types requires async code
    pub async fn new<W:HasWindowHandle + HasDisplayHandle + Sync>(window: &'window W, settings: &DisplaySettings) -> Box<dyn GraphicsEngine + 'window> {
        use wgpu::PipelineCompilationOptions;

        let window_size = settings.window_size; //window.inner_size();

        // create a wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL, // | wgpu::Backends::GL,
            flags: wgpu::InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            dx12_shader_compiler: Default::default(),
        });

        // create the serface
        let surface: wgpu::Surface<'window> = instance.create_surface(window).unwrap();

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
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                #[cfg(not(feature="texture_arrays"))]
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        // no more comments good luck!
        let surface_caps = surface.get_capabilities(&adapter);
        let present_modes = surface_caps.present_modes
            .into_iter()
            .map(VsyncUtils::map_to_vsync)
            .chain([Vsync::AutoNoVsync, Vsync::AutoVsync])
            .collect();

        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, // | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: window_size[0] as u32,
            height: window_size[1] as u32,
            present_mode: wgpu::PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],

            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            #[cfg(feature="texture_arrays")] source: wgpu::ShaderSource::Wgsl(tataku_resources::shaders::SHADER_TEX_ARRAY.into()),
            #[cfg(not(feature="texture_arrays"))] source: wgpu::ShaderSource::Wgsl(tataku_resources::shaders::SHADER.into()),
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
                    count: std::num::NonZeroU32::new(LAYER_COUNT),
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

        let window_size = Vector2::new(window_size[0], window_size[1]);
        let projection_matrix = Self::create_projection(window_size);
        let projection_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projection Matrix Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&projection_matrix.to_raw()),
        });

        let projection_matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
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
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &projection_matrix_bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });


        let mut pipelines = HashMap::new();
        for blend_mode in [
            BlendMode::AlphaBlending,
            BlendMode::AlphaOverwrite,
            BlendMode::PremultipliedAlpha,
            BlendMode::AdditiveBlending,
            BlendMode::SourceAlphaBlending,
        ] {
            let blend_state = Self::map_blend_mode(blend_mode);

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{blend_mode:?} Pipeline")),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[ Vertex::desc() ],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
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

        // create slider pipeline
        pipelines.insert(BlendMode::Slider, create_slider_pipeline(&device, &config, &projection_matrix_bind_group_layout));

        // create flashlight pipeline
        pipelines.insert(BlendMode::Flashlight, create_flashlight_pipeline(&device, &config, &projection_matrix_bind_group_layout));


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

        let particle_system = ParticleSystem::new(&device);

        let buffer_queues = [
            (LastDrawn::Slider, Box::new(RenderBufferQueueType::Slider(RenderBufferQueue::new().init(&device)))),
            (LastDrawn::Vertex, Box::new(RenderBufferQueueType::Vertex(RenderBufferQueue::new().init(&device)))),
            (LastDrawn::Flashlight, Box::new(RenderBufferQueueType::Flashlight(RenderBufferQueue::new().init(&device)))),
        ].into_iter().collect();

        Box::new(Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            pipelines,
            atlas,
            atlas_texture,

            current_render_buffer: None,
            buffer_queues,
            completed_buffers: Vec::new(),

            projection_matrix,
            projection_matrix_buffer,
            projection_matrix_bind_group,
            screenshot_pending: None,
            particle_system,
            scissors: ScissorManager::default(),
            present_modes
        })
    }




    pub fn render_current_surface(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let size = output.texture.size();

        // don't draw if our draw surface has no area
        if size.width == 0 || size.height == 0 { return Ok(()) }

        self.render(RenderableSurface::new(
            &view, 
            GFX_CLEAR_COLOR, 
            Vector2::new(size.width as f32, size.height as f32),
            false,
        ))?;

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

            self.render(RenderableSurface::new(
                &view, 
                GFX_CLEAR_COLOR, 
                Vector2::new(width as f32, height as f32), 
                true
            ))?;

            self.finish_screenshot(texture, screenshot);
        }


        Ok(())
    }

    fn render(&self, renderable: RenderableSurface) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &renderable.texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(renderable.get_clear_color()),
                        store: renderable.render_target.then_some(wgpu::StoreOp::Store).unwrap_or(wgpu::StoreOp::Discard) , // must be store for render targets to work apparently
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut current_blend_mode = BlendMode::None;
            let mut current_scissor: Scissor = None;

            for i in self.completed_buffers.iter() {
                let scissor = i.get_scissor();
                if scissor != current_scissor {
                    current_scissor = scissor;
                    let [x, y, w, h] = current_scissor.unwrap_or_else(||[0.0, 0.0, renderable.size.x, renderable.size.y]);
                    if renderable.size.x - x < 0.0 || renderable.size.y - y < 0.0 { continue }

                    let x = x.clamp(0.0, renderable.size.x);
                    let y = y.clamp(0.0, renderable.size.y);

                    render_pass.set_scissor_rect(
                        x as u32,
                        y as u32,
                        w.clamp(0.0, renderable.size.x - x) as u32,
                        h.clamp(0.0, renderable.size.y - y) as u32
                    );
                }

                let blend_mode = i.get_blend_mode();
                if blend_mode != current_blend_mode {
                    current_blend_mode = blend_mode;
                    let Some(pipeline) = self.pipelines.get(&blend_mode) else {
                        error!("Pipeline not created for blend mode {current_blend_mode:?}");
                        current_blend_mode = BlendMode::None;
                        continue
                    };

                    render_pass.set_pipeline(&pipeline);
                    render_pass.set_bind_group(0, &self.projection_matrix_bind_group, &[]);

                    if let RenderBufferType::Vertex(_) = i {
                        render_pass.set_bind_group(1, &self.atlas_texture.bind_group, &[]);
                    }
                }

                if let RenderBufferType::Slider(slider) = i {
                    render_pass.set_bind_group(1, &slider.bind_group, &[])
                }
                if let RenderBufferType::Flashlight(flashlight) = i {
                    render_pass.set_bind_group(1, &flashlight.bind_group, &[])
                }

                render_pass.set_vertex_buffer(0, i.get_vertex_buffer().slice(..));
                render_pass.set_index_buffer(i.get_index_buffer().slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..i.get_used_indices() as u32, 0, 0..1);
            }

        }

        // submit will accept anything that implements IntoIter
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
impl<'w> WgpuEngine<'w> {
    fn create_texture(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, width:u32, height:u32, format: wgpu::TextureFormat) -> WgpuTexture {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let textures = (0..LAYER_COUNT).map(|_| {
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


    fn finish_screenshot(&mut self, texture: wgpu::Texture, callback: Box<dyn FnOnce((Vec<u8>, [u32;2])) + Send + Sync>) {
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

            callback((data, [w, h]));
        });
    }
}


// render code
impl<'w> WgpuEngine<'w> {

    fn dump_last_drawn(&mut self) {
        let Some(mut last_drawn) = std::mem::take(&mut self.current_render_buffer) else { return };
        if let Some(b) = last_drawn.dump_and_next(&self.queue, &self.device) { self.completed_buffers.push(b); };
        self.buffer_queues.insert(last_drawn.draw_type(), last_drawn);
    }

    fn check_dump_and_next(&mut self, to_draw: LastDrawn) {
        if let Some(last_drawn) = &self.current_render_buffer {
            if last_drawn.draw_type() == to_draw { return }
        }

        self.dump_last_drawn();
        self.current_render_buffer = Some(self.buffer_queues.remove(&to_draw).expect(&format!("buffer queue did not have a queue for type {to_draw:?}. Did you forget to create a buffer queue for it?")));
    }



    /// returns reserve data
    fn reserve_vertex(
        &mut self,
        vtx_count: u64,
        idx_count: u64,
        blend_mode: BlendMode
    ) -> Option<VertexReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastDrawn::Vertex);

        let vertex_buffer_queue = get_render_buffer!(self, Vertex);
        // if let Some(RenderBufferQueueType::Vertex(b)) = &mut self.last_drawn {b} else {panic!("wrong buffer type")};

        let mut recording_buffer = vertex_buffer_queue.recording_buffer().expect("didnt get vertex recording buffer");
        let blend_mode_check = recording_buffer.blend_mode == blend_mode || recording_buffer.blend_mode == BlendMode::None;
        let scissor_check = recording_buffer.scissor == Some(scissor) || recording_buffer.scissor == None;
        // if !blend_mode_check { info!("blend mode changed from {:?} to {blend_mode:?}", recording_buffer.blend_mode) }

        if !blend_mode_check
        || !scissor_check
        || recording_buffer.used_vertices + vtx_count > VertexBuffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > VertexBuffer::IDX_PER_BUF {
            if let Some(b) = vertex_buffer_queue.dump_and_next(&self.queue, &self.device) {
                self.completed_buffers.push(RenderBufferType::Vertex(b))
            }

            recording_buffer = vertex_buffer_queue.recording_buffer()?;
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

        let used_vertices = recording_buffer.used_vertices;
        let used_indices = recording_buffer.used_indices;

        let cache = &mut vertex_buffer_queue.cpu_cache;
        Some(VertexReserveData {
            vtx: &mut cache.cpu_vtx[(used_vertices - vtx_count) as usize .. used_vertices as usize],
            idx: &mut cache.cpu_idx[(used_indices - idx_count) as usize .. used_indices as usize],
            idx_offset: used_vertices - vtx_count,
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
        blend_mode: BlendMode,
    ) {
        // let Some(mut reserved) = self.reserve_vertex(4, 6, scissor, blend_mode) else { return };
        let Some(mut reserved) = self.reserve_vertex(4, 6, blend_mode) else { return };

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
        blend_mode: BlendMode,
    ) {
        // let Some(mut reserved) = self.reserve_vertex(4, 6, scissor, blend_mode) else { return };
        let Some(mut reserved) = self.reserve_vertex(4, 6, blend_mode) else { return };
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

    fn reserve_slider(
        &mut self,

        slider_grid_count: u64,
        grid_cell_count: u64,
        line_segment_count: u64,
    ) -> Option<SliderReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastDrawn::Slider);

        let slider_buffer_queue = get_render_buffer!(self, Slider);
        // if let Some(RenderBufferQueueType::Slider(b)) = &mut self.last_drawn {b} else {panic!("wrong buffer type")};

        let mut recording_buffer = slider_buffer_queue.recording_buffer().expect("didnt get slider recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor) || recording_buffer.scissor == None;


        let vtx_count = 4;
        let idx_count = 6;

        if !scissor_check
        || recording_buffer.used_vertices + vtx_count > SliderRenderBuffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > SliderRenderBuffer::IDX_PER_BUF
        || recording_buffer.used_slider_data + 1 > EXPECTED_SLIDER_COUNT
        || recording_buffer.used_slider_grids + slider_grid_count > SLIDER_GRID_COUNT
        || recording_buffer.used_grid_cells + grid_cell_count > GRID_CELL_COUNT
        || recording_buffer.used_line_segments + line_segment_count > LINE_SEGMENT_COUNT
        {
            if let Some(b) = slider_buffer_queue.dump_and_next(&self.queue, &self.device) {
                self.completed_buffers.push(RenderBufferType::Slider(b))
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

        let used_vertices = recording_buffer.used_vertices;
        let used_indices = recording_buffer.used_indices;

        let used_slider_grids = recording_buffer.used_slider_grids;
        let used_grid_cells = recording_buffer.used_grid_cells;
        let used_line_segments = recording_buffer.used_line_segments;

        // reserve slider vertex data
        let slider_index = recording_buffer.used_slider_data - 1;

        let cache = &mut slider_buffer_queue.cpu_cache;
        Some(SliderReserveData {
            vtx: &mut cache.cpu_vtx[(used_vertices - vtx_count) as usize .. used_vertices as usize],
            idx: &mut cache.cpu_idx[(used_indices - idx_count) as usize .. used_indices as usize],

            slider_data: &mut cache.slider_data[slider_index as usize],
            slider_grids: &mut cache.slider_grids[(used_slider_grids - slider_grid_count) as usize .. used_slider_grids as usize],
            grid_cells: &mut cache.grid_cells[(used_grid_cells - grid_cell_count) as usize .. used_grid_cells as usize],
            line_segments: &mut cache.line_segments[(used_line_segments - line_segment_count) as usize .. used_line_segments as usize],

            idx_offset: used_vertices - vtx_count,
            slider_index: slider_index as u32,
            slider_grid_offset: (used_slider_grids - slider_grid_count) as u32,
            grid_cell_offset: (used_grid_cells - grid_cell_count) as u32,
            line_segment_offset: (used_line_segments - line_segment_count) as u32,
        })
    }


    fn reserve_flashlight(
        &mut self,
    ) -> Option<FlashlightReserveData> {
        let scissor = self.scissors.current_scissor();
        self.check_dump_and_next(LastDrawn::Flashlight);

        let buffer_queue = get_render_buffer!(self, Flashlight);
        // if let Some(RenderBufferQueueType::Slider(b)) = &mut self.last_drawn {b} else {panic!("wrong buffer type")};

        let mut recording_buffer = buffer_queue.recording_buffer().expect("didnt get flashlight recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor) || recording_buffer.scissor == None;

        let vtx_count = 4;
        let idx_count = 6;

        if !scissor_check
        || recording_buffer.used_vertices + vtx_count > SliderRenderBuffer::VTX_PER_BUF
        || recording_buffer.used_indices + idx_count > SliderRenderBuffer::IDX_PER_BUF
        {
            if let Some(b) = buffer_queue.dump_and_next(&self.queue, &self.device) {
                self.completed_buffers.push(RenderBufferType::Flashlight(b))
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
}


// draw helpers
impl<'w> WgpuEngine<'w> {
    pub fn map_blend_mode(blend_mode: BlendMode) -> wgpu::BlendState {
        match blend_mode {
            BlendMode::AlphaBlending => wgpu::BlendState::ALPHA_BLENDING,
            BlendMode::AlphaOverwrite => wgpu::BlendState::REPLACE,
            BlendMode::PremultipliedAlpha => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            BlendMode::AdditiveBlending => wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
            },
            BlendMode::SourceAlphaBlending => wgpu::BlendState {
                color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
                alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }
            },

            BlendMode::None
            | BlendMode::Slider
            | BlendMode::Flashlight => unimplemented!("nope")
        }
    }

    fn tessellate_polygon(&mut self, polygon: &Vec<Vector2>, color: Color, border: Option<f32>, transform: Matrix, blend_mode: BlendMode) {
        let mut polygon = polygon.iter();
        let mut path = lyon_tessellation::path::Path::builder();
        path.begin(polygon.next().map(|p|Point::new(p.x, p.y)).unwrap());
        for p in polygon {
            if !p.x.is_normal() || !p.y.is_normal() { return }
            path.line_to(Point::new(p.x, p.y));
        }
        path.end(true);
        let path = path.build();

        self.tessellate_path(&path, color, border, transform, blend_mode)
    }

    fn tessellate_path(&mut self, path: &lyon_tessellation::path::Path, color: Color, border: Option<f32>, transform: Matrix, blend_mode: BlendMode) {
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

        // let mut reserved = self.reserve_vertex(buffers.vertices.len() as u64, buffers.indices.len() as u64, scissor, blend_mode).expect("nope");
        let mut reserved = self.reserve_vertex(buffers.vertices.len() as u64, buffers.indices.len() as u64, blend_mode).expect("nope");

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



impl<'w> GraphicsEngine for WgpuEngine<'w> {

    fn resize(&mut self, [width, height]: [u32; 2]) {
        if width == 0 || height == 0 { return }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let window_size = Vector2::new(width as f32, height as f32);
        self.projection_matrix = Self::create_projection(window_size);
        self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&self.projection_matrix.to_raw()));
    }

    fn set_vsync(&mut self, vsync: Vsync) {
        self.config.present_mode = VsyncUtils::map_from_vsync(VsyncUtils::to_okay(vsync, &self.present_modes));
        self.surface.configure(&self.device, &self.config);
    }



    fn create_render_target(&mut self, [w, h]: [u32; 2], clear_color: Color, do_render: Box<dyn FnOnce(&mut dyn GraphicsEngine, Matrix)>) -> Option<RenderTarget> {
        // find space in the render target atlas
        let atlased = self.atlas.try_insert(w, h)?;

        // create a projection and render target
        let projection = Self::create_projection(Vector2::new(w as f32, h as f32));
        let target = RenderTarget::new_main_thread(w, h, Arc::new(atlased), projection, clear_color);

        // queue rendering the data to it
        self.update_render_target(target.clone(), do_render);

        // return the new render target
        Some(target)
    }
    fn update_render_target(&mut self, target: RenderTarget, do_render: Box<dyn FnOnce(&mut dyn GraphicsEngine, Matrix)>) {
        // get the texture this target was written to
        let textures = self.atlas_texture.textures.clone();
        let Some((atlas_tex, _)) = textures.get(target.image.tex.layer as usize) else { return };

        // write the projection matrix
        self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&target.projection.to_raw()));
        self.queue.submit([]);

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
        let renderable = RenderableSurface::new(
            &view, 
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
        if let Err(e) = self.render(renderable) {
            error!("Error rendering render target: {e:?}")
        }


        // copy render to atlas
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("render_target copy encoder") });

        let mut dest = atlas_tex.as_image_copy();
        dest.origin.x = target.image.tex.x;
        dest.origin.y = target.image.tex.y;

        encoder.copy_texture_to_texture(texture.as_image_copy(), dest, Extent3d { width, height, depth_or_array_layers: 1 });
        self.queue.submit([encoder.finish()]);

        // remove temp texture
        self.queue.on_submitted_work_done(move || texture.destroy());

        // reapply the window projection matrix
        self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&self.projection_matrix.to_raw()));

    }


    fn load_texture_bytes(&mut self, data: &[u8]) -> TatakuResult<TextureReference> {
        let diffuse_image = image::load_from_memory(data).map_err(|e| TatakuError::String(e.to_string()))?;
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let (width, height) = diffuse_image.dimensions();

        self.load_texture_rgba(&diffuse_rgba.to_vec(), [width, height])
    }

    fn load_texture_rgba(&mut self, data: &Vec<u8>, [width, height]: [u32; 2]) -> TatakuResult<TextureReference> {
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
        self.atlas.remove_entry(tex);
    }

    fn screenshot(&mut self, callback: Box<dyn FnOnce((Vec<u8>, [u32; 2]))+Send+Sync>) {
        self.screenshot_pending = Some(Box::new(callback));
    }



    fn begin_render(&mut self) {
        // if self.last_drawn is not None at this point, something went wrong
        assert!(self.current_render_buffer.is_none());

        let mut vertex_buffers = Vec::new();
        let mut slider_buffers = Vec::new();
        let mut flashlight_buffers = Vec::new();

        for i in std::mem::take(&mut self.completed_buffers) {
            match i {
                RenderBufferType::Vertex(v) => vertex_buffers.push(v),
                RenderBufferType::Slider(s) => slider_buffers.push(s),
                RenderBufferType::Flashlight(f) => flashlight_buffers.push(f),
            }
        }

        for i in self.buffer_queues.values_mut() {
            match &mut **i {
                RenderBufferQueueType::Slider(s) => s.begin(std::mem::take(&mut slider_buffers)),
                RenderBufferQueueType::Vertex(v) => v.begin(std::mem::take(&mut vertex_buffers)),
                RenderBufferQueueType::Flashlight(f) => f.begin(std::mem::take(&mut flashlight_buffers)),
            }
        }
    }

    fn end_render(&mut self) {
        if let Some(mut last_queue) = std::mem::take(&mut self.current_render_buffer) {
            if let Some(b) = last_queue.end(&self.queue) {
                self.completed_buffers.push(b);
            }

            self.buffer_queues.insert(last_queue.draw_type(), last_queue);
        }
    }

    fn present(&mut self) -> TatakuResult<()> {
        self.render_current_surface().map_err(|e| TatakuError::String(e.to_string()))
    }


    fn push_scissor(&mut self, scissor: [f32; 4]) {
        self.scissors.push_scissor(scissor)
    }
    fn pop_scissor(&mut self) {
        self.scissors.pop_scissor()
    }

    // draw helpers

    /// draw an arc with the center at 0,0
    fn draw_arc(&mut self, start: f32, end: f32, radius: f32, color: Color, resolution: u32, transform: Matrix, blend_mode: BlendMode) {
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

        self.tessellate_path(&path, color, None, transform, blend_mode);
    }

    fn draw_circle(&mut self, radius: f32, color: Color, border: Option<Border>, resolution: u32, transform: Matrix, blend_mode: BlendMode) {
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
            self.tessellate_polygon(&points, color, None, transform, blend_mode);
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

            self.tessellate_polygon(&points, border.color, Some(border.radius), transform, blend_mode);
        }

    }

    fn draw_line(&mut self, p2: Vector2, thickness: f32, color: Color, transform: Matrix, blend_mode: BlendMode) {
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
    fn draw_rect(&mut self, rect: [f32; 4], border: Option<Border>, shape: Shape, color: Color, transform: Matrix, blend_mode: BlendMode) {
        // for some reason something gets set to infinity on screen resize and panics the tesselator, this prevents that
        if rect.iter().any(|n|!n.is_normal() && *n != 0.0) { return }

        let [x, y, w, h] = rect;
        let rect = Box2D::new(Point::new(x, y), Point::new(x+w, y+h));

        let mut path = lyon_tessellation::path::Path::builder();
        match shape {
            Shape::Square => path.add_rectangle(&rect, lyon_tessellation::path::Winding::Positive),
            Shape::Round(radius) => path.add_rounded_rectangle(&rect, &BorderRadii::new(radius), lyon_tessellation::path::Winding::Positive),
            Shape::RoundSep([top_left, top_right, bottom_left, bottom_right]) => path.add_rounded_rectangle(&rect, &BorderRadii {
                top_left, top_right, bottom_left, bottom_right
            }, lyon_tessellation::path::Winding::Positive)
        }
        let path = path.build();

        // fill
        if color.a > 0.0 {
            self.tessellate_path(&path, color, None, transform, blend_mode)
        }

        // border
        if let Some(border) = border.filter(|b|b.color.a > 0.0) {
            self.tessellate_path(&path, border.color, Some(border.radius), transform, blend_mode)
        }
    }

    fn draw_tex(&mut self, tex: &TextureReference, color: Color, h_flip: bool, v_flip: bool, transform: Matrix, blend_mode: BlendMode) {
        let rect = [0.0, 0.0, tex.width as f32, tex.height as f32];
        self.reserve_tex_quad(&tex, rect, color, h_flip, v_flip, transform, blend_mode);
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
            error!("couldnt reserve slider?");
            return
        };

        // info!("{} : {} : {}", reserved.slider_grid_offset, reserved.grid_cell_offset, reserved.line_segment_offset);


        let vertices = quad.into_iter().map(|p| SliderVertex {
            position: transform.mul_v2(p).into(),
            slider_index: reserved.slider_index,
        }).collect::<Vec<_>>();

        let offset = reserved.idx_offset as u32;
        slider_data.grid_index += reserved.slider_grid_offset;
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
            &slider_grids.into_iter().map(|mut a| {a.index += reserved.grid_cell_offset; a.into()}).collect::<Vec<_>>(),
            &grid_cells.into_iter().map(|i|i + reserved.line_segment_offset).collect::<Vec<_>>(),
            &line_segments.into_iter().map(|a| a.into()).collect::<Vec<_>>()
        );
    }

    fn draw_flashlight(
        &mut self,
        quad: [Vector2; 4],
        transform: Matrix,
        flashlight_data: FlashlightData
    ) {
        let Some(mut reserved) = self.reserve_flashlight() else { return };

        let vertices = quad.into_iter().map(|p| FlashlightVertex {
            position: transform.mul_v2(p).into(),
            flashlight_index: reserved.flashlight_index,
        }).collect::<Vec<_>>();

        reserved.copy_in(&vertices, flashlight_data);
    }


    // particle engine stuff
    fn add_emitter(&mut self, emitter: Box<dyn EmitterReference>) {
        self.particle_system.add(emitter);
    }

    fn update_emitters(&mut self) {
        self.particle_system.update(&self.device, &self.queue);
    }
}


struct VsyncUtils;
impl VsyncUtils {

    fn map_to_vsync(present_mode: wgpu::PresentMode) -> Vsync {
        match present_mode {
            wgpu::PresentMode::AutoVsync => Vsync::AutoVsync,
            wgpu::PresentMode::AutoNoVsync => Vsync::AutoNoVsync,
            wgpu::PresentMode::Fifo => Vsync::Fifo,
            wgpu::PresentMode::FifoRelaxed => Vsync::FifoRelaxed,
            wgpu::PresentMode::Immediate => Vsync::Immediate,
            wgpu::PresentMode::Mailbox => Vsync::Mailbox,
        }
    }
    fn map_from_vsync(vsync: Vsync) -> wgpu::PresentMode {
        match vsync {
            Vsync::AutoVsync => wgpu::PresentMode::AutoVsync,
            Vsync::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
            Vsync::Fifo => wgpu::PresentMode::Fifo,
            Vsync::FifoRelaxed => wgpu::PresentMode::FifoRelaxed,
            Vsync::Immediate => wgpu::PresentMode::Immediate,
            Vsync::Mailbox => wgpu::PresentMode::Mailbox,
        }
    }

    fn to_okay(vsync: Vsync, present_modes: &Vec<Vsync>, ) -> Vsync {
        if Self::is_okay(&vsync, present_modes) {
            vsync
        } else {
            Self::get_fallback(vsync)
        }
    }
    fn is_okay(vsync: &Vsync, present_modes: &Vec<Vsync>) -> bool {
        present_modes.contains(vsync)
    }
    fn get_fallback(vsync: Vsync) -> Vsync {
        match vsync {
            Vsync::AutoVsync 
            | Vsync::Fifo
            | Vsync::FifoRelaxed
                => Vsync::AutoVsync,

            Vsync::AutoNoVsync 
            | Vsync::Immediate
            | Vsync::Mailbox
                => Vsync::AutoNoVsync,
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

/// pad `num` to align with `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`
fn align(num: u32) -> u32 {
    let m = num % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

    if m == 0 {
        num
    } else {
        num + (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - m)
    }
}


