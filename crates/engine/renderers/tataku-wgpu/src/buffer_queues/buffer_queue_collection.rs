use crate::prelude::*;

use tataku::MatrixHelpers as _;
use tataku::Take as _;

macro_rules! get_render_buffer {
    ($self: ident, $t: ident) => {{
        let b = $self.current_render_buffer
            .as_mut()
            .expect("last drawn type not set");

        if let RenderBufferQueueType::$t(b2) = &mut **b {b2}
        else { panic!("wrong buffer type") }
    }}
}


pub(crate) struct BufferQueueCollection {
    queue: wgpu::Queue,
    device: wgpu::Device,

    pub buffer_queues: Vec<Option<Box<RenderBufferQueueType>>>, //HashMap<PipelineType, Box<RenderBufferQueueType>>,
    pub completed_buffers: Vec<RenderBufferType>,
    pub current_render_buffer: Option<Box<RenderBufferQueueType>>,
}

impl BufferQueueCollection {
    pub fn new(
        queue: wgpu::Queue,
        device: wgpu::Device,
        pipelines: &crate::PipelineCollection,
    ) -> Self {
        let mut buffer_queues = pipelines
            .init_buffer_queues(&device)
            ;
        
        buffer_queues
            .sort_by_key(|i| i.0 as u8);
        let buffer_queues = buffer_queues.into_iter()
            .map(|(_, i)| Some(i))
            .collect();


        Self {
            queue,
            device,

            buffer_queues,
            completed_buffers: Vec::new(),
            current_render_buffer: None,
        }
    }


    pub fn dump_last_drawn(
        &mut self,
        pipelines: &crate::PipelineCollection,
    ) {
        let Some(mut last_buffer) = self
            .current_render_buffer.take()
        else { return };

        let pipeline = pipelines.get_pipeline_reference(
            last_buffer.graphics_pipeline()
        );
        if let Some(b) = last_buffer.dump_and_next(
            &self.queue,
            &self.device,
            pipeline
        ) {
            self.completed_buffers.push(b);
        };

        let i = last_buffer.pipeline_type() as usize;
        self.buffer_queues[i] = Some(last_buffer);
    }


    fn check_dump_and_next(
        &mut self, 
        to_draw: PipelineType,

        pipelines: &crate::PipelineCollection,
    ) {
        if let Some(last_buffer) = &self.current_render_buffer
            && last_buffer.pipeline_type() == to_draw
        { return }

        self.dump_last_drawn(pipelines);
        self.current_render_buffer = Some(self
            .buffer_queues[to_draw as usize]
            .take()
            .unwrap_or_else(|| panic!("buffer queue did not have a queue for type {to_draw:?}. Did you forget to create a buffer queue for it?"))
        );
    }

    pub fn reset_completed(&mut self) {

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

        for i in &mut self.buffer_queues {
            let Some(i) = i else { panic!("None in buffer queues??") };
            match &mut **i {
                RenderBufferQueueType::Slider(s) => s.reset(slider_buffers.take()),
                RenderBufferQueueType::Standard(v) => v.reset(standard_buffers.take()),
                RenderBufferQueueType::Flashlight(f) => f.reset(flashlight_buffers.take()),
                RenderBufferQueueType::GaussianBlur(f) => f.reset(gaussian_blur_buffers.take()),
                RenderBufferQueueType::BoxBlur(f) => f.reset(box_blur_buffers.take()),

                #[cfg(feature="vello")]
                RenderBufferQueueType::Vello(b) => b.begin(vello_buffers.take()),
            }
        }
    }

    pub fn begin_render(&mut self) {
        // if self.last_drawn is not None at this point, something went wrong
        assert!(self.current_render_buffer.is_none());
        self.reset_completed();
    }

    pub fn end_render(&mut self) {
        let Some(mut last_queue) = self.current_render_buffer.take()
        else { return };

        if let Some(b) = last_queue.end(&self.queue) {
            self.completed_buffers.push(b);
        }

        let i = last_queue.pipeline_type() as usize;
        self.buffer_queues[i] = Some(last_queue);
    }

}


// render code
impl BufferQueueCollection {
    /// returns reserve data
    pub fn reserve_standard<'a>(
        &'a mut self,
        vtx_count: u64,
        idx_count: u64,
        blend_mode: tataku::BlendMode,

        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) -> Option<shaders::standard::ReserveData<'a>> {
        use crate::shaders::standard;

        let scissor = scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Standard, pipelines);

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
            let pipeline = pipelines
                .get(tataku::GraphicsPipeline::Standard(blend_mode));

            if let Some(b) = vertex_buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                WgpuPipeline::Render(pipeline)
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
    pub fn reserve_tex_quad(
        &mut self,
        tex: &tataku::TextureReference,
        rect: [f32; 4],
        color: tataku::Color,
        h_flip: bool,
        v_flip: bool,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,

        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) {
        use shaders::standard;
        let Some(mut reserved) = self.reserve_standard(
            4,
            6,
            blend_mode,

            scissors,
            pipelines,
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
    pub fn reserve_quad(
        &mut self,
        quad: [tataku::Vector2; 4],
        color: tataku::Color,
        transform: tataku::Matrix,
        blend_mode: tataku::BlendMode,


        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) {
        let Some(mut reserved) = self.reserve_standard(
            4,
            6,
            blend_mode,
            
            scissors,
            pipelines,
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

    pub fn reserve_slider<'a>(
        &'a mut self,
        slider_grid_count: u64,
        grid_cell_count: u64,
        line_segment_count: u64,


        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) -> Option<shaders::slider::ReserveData<'a>> {
        use shaders::slider;

        let scissor = scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Slider, pipelines);

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
            let pipeline = pipelines
                .get(tataku::GraphicsPipeline::Slider).into();

            if let Some(b) = slider_buffer_queue.dump_and_next(
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

    pub fn reserve_flashlight<'a>(
        &'a mut self,

        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) -> Option<shaders::flashlight::ReserveData<'a>> {
        use shaders::flashlight;
        let scissor = scissors.current_scissor();
        self.check_dump_and_next(PipelineType::Flashlight, pipelines);

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
            let pipeline = pipelines
                .get(tataku::GraphicsPipeline::Flashlight).into();

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

    pub fn reserve_gaussian_blur<'a>(
        &'a mut self,

        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) -> Option<shaders::gaussian_blur::ReserveData<'a>> {
        use shaders::gaussian_blur as gaussian;
        let scissor = scissors.current_scissor();
        self.check_dump_and_next(PipelineType::GaussianBlur, pipelines);

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
                (&pipelines.gaussian_blur.pipeline).into()
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

    pub fn reserve_box_blur<'a>(
        &'a mut self,

        scissors: &tataku::ScissorManager,
        pipelines: &crate::PipelineCollection,
    ) -> Option<shaders::box_blur::ReserveData<'a>> {
        let scissor = scissors.current_scissor();
        self.check_dump_and_next(PipelineType::BoxBlur, pipelines);

        let buffer_queue = get_render_buffer!(self, BoxBlur);

        let mut recording_buffer = buffer_queue.recording_buffer()
            .expect("didnt get blur recording buffer");
        let scissor_check = recording_buffer.scissor == Some(scissor)
            || recording_buffer.scissor.is_none();

        if !scissor_check || recording_buffer.used {
            if let Some(b) = buffer_queue.dump_and_next(
                &self.queue,
                &self.device,
                (&pipelines.box_blur.pipeline).into(),
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
    pub fn reserve_vello<'a>(
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