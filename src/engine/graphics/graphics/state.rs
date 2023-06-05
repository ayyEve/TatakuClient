use crate::prelude::*;

use lyon_tessellation::math::Point;
use raw_window_handle:: {
    HasRawWindowHandle,
    HasRawDisplayHandle
};
use wgpu::{BufferBinding, util::DeviceExt, TextureViewDimension};
// use wgpu::util::DeviceExt;
use winit::window::Window;

pub struct GraphicsState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    
    render_pipeline: wgpu::RenderPipeline,
    
    recorded_buffers: Vec<RenderBuffer>,
    queued_buffers: Vec<RenderBuffer>,
    recording_buffer: Option<RenderBuffer>,
    
    //The in-progress CPU side buffers that get uploaded to the GPU upon a call to dump()
    cpu_vtx: Vec<Vertex>,
    cpu_idx: Vec<u32>,


    // texture_bind_group: wgpu::BindGroup,
    projection_matrix_bind_group: wgpu::BindGroup,
    projection_matrix_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    sampler: wgpu::Sampler,

    atlas: Atlas,
}
impl GraphicsState {
    // Creating some of the wgpu types requires async code
    async fn new(window: impl HasRawWindowHandle + HasRawDisplayHandle, settings: &Settings, size: [u32;2]) -> Self {
        let window_size = settings.window_size;
        let window_size = Vector2::new(window_size[0], window_size[1]);

        // create a wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        // create device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER | wgpu::Features::TEXTURE_BINDING_ARRAY,
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
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size[0],
            height: size[1],
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/shader.wgsl").into()),
        });


        // let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("Texture/Sampler bind group layout"),
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Texture {
        //                 multisampled: false,
        //                 view_dimension: wgpu::TextureViewDimension::D2Array,
        //                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //             },
        //             count: None,
        //         },
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 1,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             // This should match the filterable field of the
        //             // corresponding Texture entry above.
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         },
        //     ]
        // });
        
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atlas group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        // view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    // count: None,
                    // count: std::num::NonZeroU32::new(layers),
                    count: std::num::NonZeroU32::new(3),
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
                        min_binding_size: Some(unsafe{std::num::NonZeroU64::new_unchecked(proj_matrix_size)})
                    },
                    count: None,
                },
            ]
        });

        let projection = Self::create_projection(window_size);
        let projection_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Projection Matrix Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&projection.to_raw()),
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
                            size: Some(unsafe{std::num::NonZeroU64::new_unchecked(proj_matrix_size)})
                        }),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        );


        // let x = WgpuTexture::load_texture(&device, &queue, &texture_bind_group_layout);


        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &projection_matrix_bind_group_layout,
                &texture_bind_group_layout, 
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
                cull_mode: Some(wgpu::Face::Back),
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
            // mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }); 


        let atlas_size = device.limits().max_texture_dimension_2d.min(4096);
        let atlas_layers = 3;
        let atlas_texture = Self::create_texture(&device, &texture_bind_group_layout, &sampler, atlas_size, atlas_size, atlas_layers);


        let atlas = Atlas::new(
            atlas_size,
            atlas_size,
            atlas_layers,
            atlas_texture
        );

        let mut s = Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            sampler,
            atlas,

            recorded_buffers: Vec::with_capacity(3),
            queued_buffers: Vec::with_capacity(3),
            recording_buffer: None,
            cpu_vtx: vec![Vertex::default(); Self::VTX_PER_BUF as usize],
            cpu_idx: vec![0; Self::IDX_PER_BUF as usize],


            // texture_bind_group: atlas_texture.bind_group,
            projection_matrix_bind_group,
            projection_matrix_buffer,
            texture_bind_group_layout
        };
        s.create_render_buffer();

        s
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);


            let window_size = Vector2::new(new_size.width as f64, new_size.height as f64);
            let projection = Self::create_projection(window_size);
            self.queue.write_buffer(&self.projection_matrix_buffer, 0, bytemuck::cast_slice(&projection.to_raw()));
        }
    }


    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline); 
            render_pass.set_bind_group(0, &self.projection_matrix_bind_group, &[]);
            render_pass.set_bind_group(1, &self.atlas.tex.bind_group, &[]);

            for recorded_buffer in self.recorded_buffers.iter() {
                render_pass.set_vertex_buffer(0, recorded_buffer.vertex_buffer.slice(..));
                render_pass.set_index_buffer(recorded_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..recorded_buffer.used_indices as u32, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


    fn create_projection(window_size: Vector2) -> Matrix {
        let sx = (2.0 / window_size.x) as f32;
        let sy = (-2.0 / window_size.y) as f32;
        
        [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0]
        ].into()
    }

}

// texture stuff
impl GraphicsState {
    fn create_texture(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler, width:u32, height:u32, layers: u32) -> WgpuTexture {
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let textures = (0..layers).map(|_| {
            let texture = device.create_texture(
                &wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    // Most images are stored using sRGB so we need to reflect that here.
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: Some("texture_atlas"),
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
                label: Some("pain and suffering"),
                dimension: Some(TextureViewDimension::D2Array),
                base_array_layer: 0,
                // array_layer_count: Some(3),

                ..Default::default()
            });
            (texture, view)
        }).collect::<Vec<_>>();

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
                },
                // wgpu::BindGroupEntry {
                //     binding: 2,
                //     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                //         buffer: &texture_index_buffer,
                //         offset: 0,
                //         size: Some(NonZeroU64::new(4).unwrap()),
                //     }),
                // },
            ],
        });

        // let bind_group = device.create_bind_group(
        //     &wgpu::BindGroupDescriptor {
        //         layout: &layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureView(&texture_view),
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(sampler),
        //             }
        //         ],
        //         label: Some("diffuse_bind_group"),
        //     }
        // );
        

        WgpuTexture {
            textures,
            // texture,
            // texture_view,
            bind_group
        }
    }


    pub fn load_texture_path(&mut self, tex_name: &String, file_path: impl AsRef<Path>) -> TatakuResult<TextureReference> {
        let bytes = std::fs::read(file_path.as_ref())?;
        self.load_texture_bytes(tex_name, bytes)
    }

    pub fn load_texture_bytes(&mut self, tex_name: &String, data: impl AsRef<[u8]>) -> TatakuResult<TextureReference> {
        let diffuse_image = image::load_from_memory(data.as_ref())?;
        let diffuse_rgba = diffuse_image.to_rgba8();

        use image::GenericImageView;
        let (width, height) = diffuse_image.dimensions();

        self.load_texture_rgba(tex_name, &diffuse_rgba.to_vec(), width, height)
    }

    pub fn load_texture_rgba(&mut self, tex_name: &String, data: &Vec<u8>, width: u32, height: u32) -> TatakuResult<TextureReference> {
        let Some(info) = self.atlas.try_insert(tex_name, width, height) else { return Err(TatakuError::String("no space in atlas".to_owned())); };
        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        self.queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.atlas.tex.textures.get(info.layer as usize).unwrap().0,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: info.x,
                    y: info.y,
                    z: 0
                },
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            data,
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

}


// render code
impl GraphicsState {
    const QUAD_PER_BUF:u64 = 5000;
    const VTX_PER_BUF:u64 = Self::QUAD_PER_BUF * 4;
    const IDX_PER_BUF:u64 = Self::QUAD_PER_BUF * 6;

    fn create_render_buffer(&mut self) {
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
            used_vertices: 0,
            used_indices: 0,
        })
    }

    fn begin(&mut self) {
        // Go through all recorded buffers, and set their used counts to 0, resetting them for the next use
        for i in self.recorded_buffers.iter_mut() {
            i.used_indices = 0;
            i.used_vertices = 0;
        }
        
        // Move all recorded buffers into the queued buffers list
        self.queued_buffers.append(&mut self.recorded_buffers);
        self.recording_buffer = Some(self.queued_buffers.pop().unwrap());
        // self.started = true;
    }

    fn end(&mut self) {
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


    fn reserve(
        &mut self,
        vtx_count: u64,
        idx_count: u64,
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
        
        Some(ReserveData {
            vtx: &mut self.cpu_vtx[(recording_buffer.used_vertices - vtx_count) as usize .. recording_buffer.used_vertices as usize],
            idx: &mut self.cpu_idx[(recording_buffer.used_indices - idx_count) as usize .. recording_buffer.used_indices as usize],
            idx_offset: recording_buffer.used_vertices - vtx_count
        })
    }


    fn reserve_tex_quad(
        &mut self,
        tex: &TextureReference,
        rect: [f32; 4],
        depth: f32,
        color: Color,
        transform: Matrix
    ) {
        let Some(mut reserved) = self.reserve(4, 6) else { return };
        
        let [x, y, w, h] = rect;
        let color = color.into();

        reserved.copy_in(&mut [
            Vertex {
                position: transform.mul_v3(Vector3::new(x, y, depth)).into(),
                tex_coords: tex.uvs.tl.into(),
                tex_index: tex.layer as i32,
                color
            },
            Vertex {
                // .position = position + (Gfx.Vector2{ size[0], 0 } * scale),
                position: transform.mul_v3(Vector3::new(x+w, y, depth)).into(),
                tex_coords: tex.uvs.tr.into(),
                tex_index: tex.layer as i32,
                color
            },
            Vertex {
                // .position = position + (Gfx.Vector2{ 0, size[1] } * scale),
                position: transform.mul_v3(Vector3::new(x, y+h, depth)).into(),
                tex_coords: tex.uvs.bl.into(),
                tex_index: tex.layer as i32,
                color
            },
            Vertex {
                //     .position = position + (size * scale),
                position: transform.mul_v3(Vector3::new(x+w, y+h, depth)).into(),
                tex_coords: tex.uvs.br.into(),
                tex_index: tex.layer as i32,
                color
            }
        ], &mut [
            0 + reserved.idx_offset as u32,
            2 + reserved.idx_offset as u32,
            1 + reserved.idx_offset as u32,
            1 + reserved.idx_offset as u32,
            2 + reserved.idx_offset as u32,
            3 + reserved.idx_offset as u32,
        ]);
    }

    fn reserve_quad(
        &mut self,
        rect: [f32; 4],
        depth: f32,
        color: Color,
        transform: Matrix
    ) {
        let Some(mut reserved) = self.reserve(4, 6) else { return };
        let tex_coords = [0.0, 0.0];
        let tex_index = -1;
        let color = color.into();

        let [x, y, w, h] = rect;
        reserved.copy_in(&mut [
            Vertex {
                position: transform.mul_v3(Vector3::new(x, y, depth)).into(),
                tex_coords,
                tex_index,
                color
            },
            Vertex {
                position: transform.mul_v3(Vector3::new(x+w, y, depth)).into(),
                tex_coords,
                tex_index,
                color
            },
            Vertex {
                position: transform.mul_v3(Vector3::new(x, y+h, depth)).into(),
                tex_coords,
                tex_index,
                color
            },
            Vertex {
                position: transform.mul_v3(Vector3::new(x+w, y+h, depth)).into(),
                tex_coords,
                tex_index,
                color
            }
        ], &mut [
            0 + reserved.idx_offset as u32,
            2 + reserved.idx_offset as u32,
            1 + reserved.idx_offset as u32,
            1 + reserved.idx_offset as u32,
            2 + reserved.idx_offset as u32,
            3 + reserved.idx_offset as u32,
        ]);
    }

}

// draw helpers
impl GraphicsState {
    pub fn draw_circle(&mut self, depth: f32, radius: f64, color: Color, border: Option<&Border>, resolution: u32, transform: Matrix) {
        if let Some(border) = border {
            self.draw_circle(depth, radius + border.radius, border.color, None, resolution, transform)
        }

        let (x, y, w, h) = (-radius, -radius, 2.0 * radius, 2.0 * radius);
        let (cw, ch) = (0.5 * w, 0.5 * h);
        let (cx, cy) = (x + cw, y + ch);
        let n = resolution;

        let points = (0..n).map(|i| {
            let angle = i as f64 / n as f64 * (PI * 2.0);
            Vector2::new(cx + angle.cos() * cw, cy + angle.sin() * ch)
        });

        self.tesselate_polygon(points, depth, color, transform);
    }

    pub fn draw_line(&mut self, line: [f64; 4], thickness: f64, depth: f32, color: Color, transform: Matrix) {
        let resolution = 2;
        let n = resolution * 2;
        
        let radius = thickness;
        let (x1, y1, x2, y2) = (line[0], line[1], line[2], line[3]);
        let (dx, dy) = (x2 - x1, y2 - y1);
        let w = (dx * dx + dy * dy).sqrt();
        let pos1 = cgmath::Vector3::new(x1 as f32, y1 as f32, 0.0);
        let d = cgmath::Vector2::new(dx as f32, dy as f32);
        
        let m = transform * Matrix::from_translation(pos1) * Matrix::from_orient(d);
        let points = (0..n).map(|j| {
            // Detect the half circle from index.
            // There is one half circle at each end of the line.
            // Together they form a full circle if
            // the length of the line is zero.
            match j {
                j if j >= resolution => {
                    // Compute the angle to match start and end
                    // point of half circle.
                    // This requires an angle offset since
                    // the other end of line is the first half circle.
                    let angle = (j - resolution) as f64 / (resolution - 1) as f64 * PI + PI;
                    // Rotate 90 degrees since the line is horizontal.
                    let angle = angle + PI / 2.0;
                    Vector2::new(w + angle.cos() * radius, angle.sin() * radius)
                }
                j => {
                    // Compute the angle to match start and end
                    // point of half circle.
                    let angle = j as f64 / (resolution - 1) as f64 * PI;
                    // Rotate 90 degrees since the line is horizontal.
                    let angle = angle + PI / 2.0;
                    Vector2::new(angle.cos() * radius, angle.sin() * radius)
                }
            }
        });

        self.tesselate_polygon(points, depth, color, m);
    }

    pub fn draw_rect(&mut self, rect: [f32; 4], depth: f32, border: Option<&Border>, color: Color, transform: Matrix) {
        self.reserve_quad(rect, depth, color, transform)
    }

    pub fn draw_tex(&mut self, tex: &TextureReference, depth: f32, color: Color, transform: Matrix) {
        let rect = [0.0, 0.0, tex.width as f32, tex.height as f32];
        self.reserve_tex_quad(&tex, rect, depth, color, transform)
    }


    fn tesselate_polygon(&mut self, polygon: impl Iterator<Item=Vector2>, depth: f32, color: Color, transform: Matrix) {
        let mut path = lyon_tessellation::path::Path::builder();

        let mut started = false;
        for p in polygon {
            if !started {
                path.begin(Point::new(p.x as f32, p.y as f32));
                started = true
            }
            path.line_to(Point::new(p.x as f32, p.y as f32));
        }
        path.end(true);
        let path = path.build();

        
        // Create the destination vertex and index buffers.
        let mut buffers: lyon_tessellation::VertexBuffers<Point, u16> = lyon_tessellation::VertexBuffers::new();

        {
            let mut vertex_builder = lyon_tessellation::geometry_builder::simple_builder(&mut buffers);

            // Create the tessellator.
            let mut tessellator = lyon_tessellation::FillTessellator::new();

            // Compute the tessellation.
            let result = tessellator.tessellate_path(
                &path,
                &lyon_tessellation::FillOptions::default(),
                &mut vertex_builder
            );
            assert!(result.is_ok());
        }

        // convert vertices and indices to their proper values
        let mut vertices = buffers.vertices.into_iter().map(|n| Vertex {
                position: [n.x, n.y, depth],
                tex_coords: [0.0, 0.0],
                tex_index: -1,
                color: [color.r, color.g, color.b, color.a]
            }.apply_matrix(&transform)
        ).collect::<Vec<_>>();
        
        // insert the vertices and indices into the render buffer
        let mut reserved = self.reserve(vertices.len() as u64, buffers.indices.len() as u64).expect("nope");
        let mut indices = buffers.indices.into_iter().map(|a|reserved.idx_offset as u32 + a as u32).collect::<Vec<_>>();
        reserved.copy_in(&mut vertices, &mut indices);
    }

}



pub struct WgpuTexture {
    textures: Vec<(wgpu::Texture, wgpu::TextureView)>,
    // texture: wgpu::Texture,
    // pub texture_view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

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

    let tex = state.load_texture_path(&"test_tex".to_owned(), "C:/Users/Eve/Desktop/Projects/rust/tataku/tataku-client/game/skins/bubbleman/default-4.png").expect("failed to load tex");
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
                Matrix::from_translation(Vector3::new(self.position.x as f32, self.position.y as f32, depth))
                * Matrix::from_nonuniform_scale(self.scale.x as f32, self.scale.y as f32, 1.0)
                * Matrix::from_angle_z(cgmath::Rad(self.rotation))
            ;
            
            // state.draw_circle(depth, 100.0, Color::GREEN, None, 100, m);

            let angle = PI / 3.0;
            let p2 = Vector2::from_angle(angle) * 100.0;
            let line = [0.0, 0.0, p2.x, p2.y];
            state.draw_line(line, 5.0, depth, Color::RED, m);


            state.draw_tex(&self.tex, depth, Color::WHITE, m);
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
                    position.x as f64,
                    position.y as f64,
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
        },

        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.begin();
            for i in things.iter_mut() {
                i.draw(&mut state)
            }
            state.end();

            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                // Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("error: {:?}", e),
            }
        },

        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        
        _ => {}
    });

}
 