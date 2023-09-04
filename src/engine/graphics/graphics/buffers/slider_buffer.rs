use wgpu::{
    Device,
    Buffer,
    BufferUsages,
};
use crate::prelude::*;

const EXPECTED_SLIDER_COUNT: u64 = 500;
const SLIDER_GRID_COUNT: u64 = EXPECTED_SLIDER_COUNT * 32;
const GRID_CELL_COUNT: u64 = EXPECTED_SLIDER_COUNT * 32 * 16;
const LINE_SEGMENT_COUNT: u64 = EXPECTED_SLIDER_COUNT * 32 * 16 * 2;

pub struct SliderRenderBuffer {
    pub circle_radius: Buffer,
    pub border_width: Buffer,

    pub slider_data: Buffer,
    pub slider_grids: Buffer,
    pub grid_cells: Buffer,
    pub line_segments: Buffer,

    pub used_slider_data: u64,
    pub used_slider_grids: u64,
    pub used_grid_cells: u64,
    pub used_line_segments: u64,
}

impl RenderBufferable for SliderRenderBuffer {
    type Cache = CpuSliderRenderBuffer;

    fn reset(&mut self) {
        self.used_slider_data = 0;
        self.used_slider_grids = 0;
        self.used_grid_cells = 0;
        self.used_line_segments = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &Self::Cache) {
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
        Self {
            used_slider_data: 0,
            used_slider_grids: 0,
            used_grid_cells: 0,
            used_line_segments: 0,

            circle_radius: create_buffer::<f32>(device, BufferUsages::UNIFORM, 1),
            border_width: create_buffer::<f32>(device, BufferUsages::UNIFORM, 1),
            slider_data: create_buffer::<SliderData>(device, BufferUsages::STORAGE, EXPECTED_SLIDER_COUNT),
            slider_grids: create_buffer::<GridCell>(device, BufferUsages::STORAGE, EXPECTED_SLIDER_COUNT * 32),
            grid_cells: create_buffer::<u32>(device, BufferUsages::STORAGE, EXPECTED_SLIDER_COUNT * 32 * 16),
            line_segments: create_buffer::<LineSegment>(device, BufferUsages::STORAGE, EXPECTED_SLIDER_COUNT * 32 * 16 * 2),
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
            circle_radius: 0.0,
            border_width: 0.0,
            slider_data: vec![Default::default(); EXPECTED_SLIDER_COUNT as usize],
            slider_grids: vec![Default::default(); SLIDER_GRID_COUNT as usize],
            grid_cells: vec![Default::default(); GRID_CELL_COUNT as usize],
            line_segments: vec![Default::default(); LINE_SEGMENT_COUNT as usize],
        }
    }
}
