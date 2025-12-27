// WARNING: there is a lot of just data and setup code in this
use std::sync::Arc;
use std::borrow::Cow;

use crate::prelude::*;
use crate::atlas::WgpuAtlas;
use crate::renderable_surface::*;
use graphics::RenderingEngine as _;

use tataku::{
    MatrixHelpers as _, 
    Interpolation as _,
};

use raw_window_handle::{ 
    HasWindowHandle, 
    HasDisplayHandle 
};

use lyon_tessellation:: {
    geom::{ Box2D, Point },
    path::{ builder::BorderRadii, Path as LyonPath },
};

/// background color
const GFX_CLEAR_COLOR:tataku::Color = tataku::Color::BLACK;

pub struct WgpuEngine<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,

    pipelines: PipelineCollection, 
    buffer_queues: BufferQueueCollection,
    particle_system: shaders::particles::ParticleSystem,
    
    atlas: WgpuAtlas,
    projection_matrix: ProjectionMatrix,

    intermediate_texture: Option<wgpu::Texture>,
    screenshot_pending: Option<graphics::ScreenshotCallback>,

    blitterer: wgpu::util::TextureBlitter,
    font_scale_context: parley::swash::scale::ScaleContext,
    
    pub(crate) scissors: tataku::ScissorManager,

    can_blur: bool,
    blur_enabled: bool,
    present_modes: Vec<tataku::Vsync>,
}
impl<'window> WgpuEngine<'window> {

    // Creating some of the wgpu types requires async code
    pub async fn create<W:HasWindowHandle + HasDisplayHandle + Sync + ?Sized>(
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
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::BGRA8UNORM_STORAGE,
                required_limits: wgpu::Limits {
                    max_binding_array_elements_per_shader_stage: WgpuAtlas::LAYER_COUNT.end,
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

        let atlas = WgpuAtlas::new(
            &device,
            surface_format,
        );

        let projection_matrix = ProjectionMatrix::new(
            tataku::Vector2::new(window_size[0], window_size[1]),
            &device,
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

        let pipelines = PipelineCollection::new(
            &device, 
            &projection_matrix,
            &atlas, 
            &WgpuTextureReference::new(&intermediate_texture),
        );
        
        let particle_system = shaders::particles::ParticleSystem::new(&device);

        // let buffer_queues = pipelines.init_buffer_queues(&device);
        let buffer_queues = BufferQueueCollection::new(
            queue.clone(),
            device.clone(),
            &pipelines,
        );

        let blitterer = wgpu::util::TextureBlitter::new(
            &device,
            surface_format,
        );

        Box::new(Self {
            surface,
            device,
            queue: Arc::new(queue),
            config,
            pipelines,
            atlas,

            buffer_queues,

            projection_matrix,
            screenshot_pending: None,

            particle_system,

            #[cfg(feature="vello_rendering")] vello_pipeline,
            font_scale_context: parley::swash::scale::ScaleContext::new(),
            blitterer,

            scissors: tataku::ScissorManager::default(),
            present_modes,
            can_blur,
            blur_enabled: true,
            intermediate_texture: Some(intermediate_texture),
        })
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
        self.blitterer.copy(
            &self.device,
            &mut encoder,
            &texture.create_view(&view),
            &swapchain.texture.create_view(&view)
        );
        
        self.queue.submit([encoder.finish()]);
        swapchain.present();

        if let Some(screenshot) = self.screenshot_pending.take() {
            let (data, size) = self.texture_to_bytes(
                &texture
            );

            screenshot((data, size));
        }

        self.atlas.clear_glyphs();

        self.intermediate_texture = Some(texture);

        Ok(())
    }

    fn render(&mut self, renderable: &RenderableSurface) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );

        // let time = std::time::Instant::now();
        // let mut list = Vec::with_capacity(self.completed_buffers.len() + 1);
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

            for i in self.buffer_queues.completed_buffers.iter() {
                let pipeline_type = i.get_pipeline_type();
                // list.push(format!(
                //     "{:.2}ms -> {pipeline_type:?}", 
                //     time.elapsed().as_secs_f32() * 1000.0
                // ));

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
                    self.pipelines.perform_compute_pipeline(
                        pipeline_type, 
                        i, 
                        &self.device, 
                        &self.queue, 
                        renderable.texture
                    );


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
                    let pipeline = self.pipelines.get(pipeline);
                    // let Some(pipeline) = self.pipelines.get(pipeline)
                    // else {
                    //     error!("Pipeline not created for blend mode {current_pipeline:?}");
                    //     current_pipeline = tataku::GraphicsPipeline::None;
                    //     continue
                    // };

                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(
                        0,
                        &self.projection_matrix.bind_group,
                        &[]
                    );

                    if let RenderBufferType::Standard(_) = i {
                        render_pass.set_bind_group(
                            1,
                            &self.atlas.bind_group,
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

            // list.push(format!("{:.2}ms: end", time.elapsed().as_secs_f32() * 1000.0));
        }
        // println!("{}", list.join("\n"));

        self.queue.submit([encoder.finish()]);

        Ok(())
    }


    pub(crate) fn texture_to_bytes(
        &self, 
        texture: &wgpu::Texture,
    ) -> (Vec<u8>, [u32;2]) {
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

    fn load_texture(
        &mut self, 
        rgba: &[u8],
        [width, height]: [u32; 2],
        glyph: bool,
    ) -> Option<tataku::TextureReference> {
        use crate::atlas::AtlasResult;

        let result = self.atlas.reserve(
            width, 
            height, 
            glyph,
            &self.device,
        );
        let info = match result {
            AtlasResult::Ok(a) => a,
            AtlasResult::Resized(a) => {
                self.pipelines.atlas_resized(
                    &self.device, 
                    &mut self.buffer_queues,
                    &self.projection_matrix, 
                    &self.atlas
                );
                a
            },
            AtlasResult::NoSpace => {
                error!("Error inserting size ({width}x{height}) into atlas!");
                return None;
            },
        };

        if info.is_empty() { return Some(info) }

        let mut data = Cow::Borrowed(rgba);

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
                texture: self.atlas.get_texture(&info),
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

        Some(info)
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

        let mut reserved = self.buffer_queues.reserve_standard(
            buffers.vertices.len() as u64,
            buffers.indices.len() as u64,
            blend_mode,

            &self.scissors,
            &self.pipelines,
        ).expect("nope");

        // convert vertices and indices to their proper values
        let vertices = buffers.vertices
            .into_iter()
            .map(|n|
                shaders::standard::Vertex {
                    position: [n.x, n.y],
                    color: color.into(),
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
        self.projection_matrix.update_projection(
            window_size, 
            &self.queue
        );
    }

    fn begin_render(&mut self) {
        self.buffer_queues.begin_render();
    }
    fn end_render(&mut self) {
        self.buffer_queues.end_render();
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
        self.atlas.dump(self, path);
    }


    fn load_texture_bytes(
        &mut self, 
        data: &[u8]
    ) -> tataku::Result<tataku::TextureReference> {
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
    ) -> tataku::Result<tataku::TextureReference> {
        self.load_texture(data, [width, height], false)
        .ok_or_else(|| tataku::Error::String(
            "no space in atlas".to_owned()
        ))
    }

    fn free_tex(
        &mut self, 
        tex: tataku::TextureReference, 
    ) {
        if tex.is_empty() { return }

        // remove from texture atlas
        self.atlas.remove(tex);
    }

    fn screenshot(&mut self, callback: graphics::ScreenshotCallback) {
        self.screenshot_pending = Some(Box::new(callback));
    }

    fn present(&mut self) -> tataku::Result<()> {
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
        self.buffer_queues.reserve_quad(
            quad, 
            color, 
            transform, 
            blend_mode,

            &self.scissors,
            &self.pipelines,
        );
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
        self.buffer_queues.reserve_tex_quad(
            tex.tex,
            [0.0, 0.0, tex.tex.width as f32, tex.tex.height as f32],
            tex.color,
            tex.flip.flip_h(),
            tex.flip.flip_v(),
            transform,
            blend_mode,

            &self.scissors,
            &self.pipelines,
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
        let Some(mut reserved) = self.buffer_queues.reserve_slider(
            slider_grids.len() as u64,
            grid_cells.len() as u64,
            line_segments.len() as u64,

            &self.scissors,
            &self.pipelines,
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
        let Some(mut reserved) = self.buffer_queues.reserve_flashlight(
            &self.scissors,
            &self.pipelines,
        )
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
        let Some(mut reserve) = self.buffer_queues.reserve_box_blur(
            &self.scissors,
            &self.pipelines,
        )
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
        let Some(mut reserve) = self.buffer_queues.reserve_gaussian_blur(
            &self.scissors,
            &self.pipelines,
        )
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
        layout: &parley::Layout<tataku_engine_common::prelude::Color>,
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
                .map(|&alpha| g.color.with_alpha(alpha.into()))
                .flat_map(<[u8; 4]>::from)
                .collect::<Vec<_>>();

            let Some(tex) = self.load_texture(
                &data, 
                size, 
                true
            )
            else { return };
            // let tex = self.load_texture_rgba(&data, size).unwrap();

            self.draw_tex(
                graphics::TextureDraw::new(
                    &tex,
                    tataku::Color::WHITE,
                ),
                transform.trans(g.pos),
                blend_mode,
            );

            // self.free_tex(tex, true);
        }

    }


    fn create_render_target(
        &mut self,
        data: &mut graphics::RenderTargetData,
        do_render: graphics::RenderTargetDraw,
    ) {
        let width = data.width;
        let height = data.height;
        // find space in the render target atlas
        let atlased = self.atlas.reserve(
            width, 
            height, 
            false,
            &self.device,
        );

        use crate::atlas::AtlasResult;
        let atlased = match atlased {
            AtlasResult::Ok(a) => a,
            AtlasResult::Resized(a) => {
                self.pipelines.atlas_resized(
                    &self.device, 
                    &mut self.buffer_queues,
                    &self.projection_matrix, 
                    &self.atlas,
                );
                a
            },
            AtlasResult::NoSpace => {
                error!("Error inserting size ({width},{height}) into atlas!");
                return
            },
        };

        // create a projection and render target
        let projection = ProjectionMatrix::create_projection(
            tataku::Vector2::new(width as f32, height as f32)
        );

        data.tex = Arc::new(atlased);
        data.projection = projection;

        // queue rendering the data to it
        self.update_render_target(data, do_render);
    }

    fn update_render_target(
        &mut self,
        data: &tataku_graphics::RenderTargetData,
        do_render: tataku_graphics::RenderTargetDraw,
    ) {
        if data.tex.is_empty() {
            return
        }

        // get the texture this target was written to
        let atlas_tex = self.atlas
            .get_texture(&data.tex)
            .clone();

        // write the projection matrix
        self.projection_matrix.write_projection(data.projection, &self.queue);
        self.queue.submit([]);

        // create a temporary texture to render this target to
        let texture = self.device.create_texture(
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: data.width,
                    height: data.height,
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
            data.clear_color,
            tataku::Vector2::new(data.width as f32, data.height as f32),
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
        dest.origin.x = data.tex.x;
        dest.origin.y = data.tex.y;

        encoder.copy_texture_to_texture(
            texture.as_image_copy(),
            dest,
            texture.size(),
        );
        self.queue.submit([encoder.finish()]);

        // remove temp texture
        self.queue.on_submitted_work_done(move || texture.destroy());

        // reapply the window projection matrix
        self.projection_matrix.reapply_projection(&self.queue);

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
