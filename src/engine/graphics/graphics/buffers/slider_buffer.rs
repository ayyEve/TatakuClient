use wgpu::{
    Device,
    Buffer,
    BufferUsages,
    BindGroup,
    BindGroupLayout, BindGroupEntry,
};
use crate::prelude::*;

const QUAD_PER_BUF:u64 = 3000;
const VTX_PER_BUF:u64 = QUAD_PER_BUF * 4;
const IDX_PER_BUF:u64 = QUAD_PER_BUF * 6;

pub const EXPECTED_SLIDER_COUNT: u64 = 500;
pub const SLIDER_GRID_COUNT: u64 = EXPECTED_SLIDER_COUNT * 32;
pub const GRID_CELL_COUNT: u64 = SLIDER_GRID_COUNT * 16;
pub const LINE_SEGMENT_COUNT: u64 = GRID_CELL_COUNT * 2;

pub static SLIDER_BIND_GROUP_LAYOUT: OnceCell<BindGroupLayout> = OnceCell::const_new();

pub struct SliderRenderBuffer {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub scissor: Option<Scissor>,

    pub circle_radius: Buffer,
    pub border_width: Buffer,

    pub slider_data: Buffer,
    pub slider_grids: Buffer,
    pub grid_cells: Buffer,
    pub line_segments: Buffer,


    pub used_vertices: u64,
    pub used_indices: u64,

    pub used_slider_data: u64,
    pub used_slider_grids: u64,
    pub used_grid_cells: u64,
    pub used_line_segments: u64,

    pub bind_group: BindGroup
}

impl RenderBufferable for SliderRenderBuffer {
    type Cache = CpuSliderRenderBuffer;

    fn reset(&mut self) {
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;

        self.used_slider_data = 0;
        self.used_slider_grids = 0;
        self.used_grid_cells = 0;
        self.used_line_segments = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&cache.cpu_vtx));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&cache.cpu_idx));

        queue.write_buffer(&self.circle_radius, 0, bytemuck::cast_slice(&[cache.circle_radius]));
        queue.write_buffer(&self.border_width, 0, bytemuck::cast_slice(&[cache.border_width]));

        queue.write_buffer(&self.slider_data, 0, bytemuck::cast_slice(&cache.slider_data));
        queue.write_buffer(&self.slider_grids, 0, bytemuck::cast_slice(&cache.slider_grids));
        queue.write_buffer(&self.grid_cells, 0, bytemuck::cast_slice(&cache.grid_cells));
        queue.write_buffer(&self.line_segments, 0, bytemuck::cast_slice(&cache.line_segments));
    }

    fn should_write(&self) -> bool {
        self.used_slider_data > 0
    }

    fn create_new_buffer(device: &Device) -> Self {
        info!("creating new slider buffer");
        let bind_group_layout = SLIDER_BIND_GROUP_LAYOUT.get().unwrap();

        let circle_radius = create_buffer::<f32>(device, BufferUsages::UNIFORM, 1);
        let border_width = create_buffer::<f32>(device, BufferUsages::UNIFORM, 1);
        let slider_data = create_buffer::<SliderData>(device, BufferUsages::STORAGE, EXPECTED_SLIDER_COUNT);
        let slider_grids = create_buffer::<GridCell>(device, BufferUsages::STORAGE, SLIDER_GRID_COUNT);
        let grid_cells = create_buffer::<u32>(device, BufferUsages::STORAGE, GRID_CELL_COUNT);
        let line_segments = create_buffer::<LineSegment>(device, BufferUsages::STORAGE, LINE_SEGMENT_COUNT);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: circle_radius.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: border_width.as_entire_binding() },
                BindGroupEntry { binding: 2, resource: slider_data.as_entire_binding() },
                BindGroupEntry { binding: 3, resource: slider_grids.as_entire_binding() },
                BindGroupEntry { binding: 4, resource: grid_cells.as_entire_binding() },
                BindGroupEntry { binding: 5, resource: line_segments.as_entire_binding() },
            ]
        });

        Self {
            scissor: None,

            used_slider_data: 0,
            used_slider_grids: 0,
            used_grid_cells: 0,
            used_line_segments: 0,
            used_vertices: 0,
            used_indices: 0,

            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: VTX_PER_BUF * std::mem::size_of::<Vertex>() as u64,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer"),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                size: IDX_PER_BUF * std::mem::size_of::<u32>() as u64,
                mapped_at_creation: false,
            }),

            circle_radius,
            border_width,
            slider_data,
            slider_grids,
            grid_cells,
            line_segments,
            bind_group,
        }
    }
}

fn create_buffer<T>(device: &Device, t: BufferUsages, count: u64) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Slider Buffer"),
        usage: t | wgpu::BufferUsages::COPY_DST,
        size: count * std::mem::size_of::<T>() as u64,
        mapped_at_creation: false,
    })
}


pub struct CpuSliderRenderBuffer {
    pub cpu_vtx: Vec<SliderVertex>,
    pub cpu_idx: Vec<u32>,

    pub circle_radius: f32,
    pub border_width: f32,
    pub slider_data: Vec<SliderData>,
    pub slider_grids: Vec<GridCell>,
    pub grid_cells: Vec<u32>,
    pub line_segments: Vec<LineSegment>,
}
impl Default for CpuSliderRenderBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![SliderVertex::default(); VTX_PER_BUF as usize],
            cpu_idx: vec![0; IDX_PER_BUF as usize],

            circle_radius: 0.0,
            border_width: 0.0,
            slider_data: vec![Default::default(); EXPECTED_SLIDER_COUNT as usize],
            slider_grids: vec![Default::default(); SLIDER_GRID_COUNT as usize],
            grid_cells: vec![Default::default(); GRID_CELL_COUNT as usize],
            line_segments: vec![Default::default(); LINE_SEGMENT_COUNT as usize],
        }
    }
}

#[derive(Debug)]
pub struct SliderReserveData<'a> {
    pub vtx: &'a mut [SliderVertex],
    pub idx: &'a mut [u32],

    pub slider_data: &'a mut SliderData,
    pub slider_grids: &'a mut [GridCell],
    pub grid_cells: &'a mut [u32],
    pub line_segments: &'a mut [LineSegment],


    pub idx_offset: u64,
    pub slider_index: u32,
    pub slider_grid_offset: u32,
    pub grid_cell_offset: u32,
    pub line_segment_offset: u32,
}

impl<'a> SliderReserveData<'a> {
    pub fn copy_in(
        &mut self, 
        vtx: &[SliderVertex], 
        idx: &[u32],

        slider_data: SliderData,
        slider_grids: &[GridCell],
        grid_cells: &[u32],
        line_segments: &[LineSegment]
    ) {
        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);
        
        *self.slider_data = slider_data;
        self.slider_grids.copy_from_slice(slider_grids);
        self.grid_cells.copy_from_slice(grid_cells);
        self.line_segments.copy_from_slice(line_segments);

        // for i in 0..slider_grids.len() { self.slider_grids[i] = slider_grids[i] }
        // for i in 0..grid_cells.len() { self.grid_cells[i] = grid_cells[i] }
        // for i in 0..line_segments.len() { self.line_segments[i] = line_segments[i] }
    }
}
