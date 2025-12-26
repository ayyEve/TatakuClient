use crate::prelude::*;
use crate::wgpu_engine::WgpuPipeline;
use crate::buffer_queue::RenderBufferable;

const QUAD_PER_BUF:u64 = 3000;
const VTX_PER_BUF:u64 = QUAD_PER_BUF * 4;
const IDX_PER_BUF:u64 = QUAD_PER_BUF * 6;

pub(crate) const EXPECTED_SLIDER_COUNT:u64 = 15;
pub(crate) const SLIDER_GRID_COUNT:u64 = EXPECTED_SLIDER_COUNT * 32;
pub(crate) const GRID_CELL_COUNT:u64 = SLIDER_GRID_COUNT * 16;
pub(crate) const LINE_SEGMENT_COUNT:u64 = GRID_CELL_COUNT * 2;

pub(crate) struct Buffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub scissor: Option<tataku::Scissor>,

    pub slider_data: wgpu::Buffer,
    pub slider_grids: wgpu::Buffer,
    pub grid_cells: wgpu::Buffer,
    pub line_segments: wgpu::Buffer,


    pub used_vertices: u64,
    pub used_indices: u64,

    pub used_slider_data: u64,
    pub used_slider_grids: u64,
    pub used_grid_cells: u64,
    pub used_line_segments: u64,

    pub bind_group: wgpu::BindGroup
}
impl RenderBufferable for Buffer {
    type Cache = CpuBuffer;
    const VTX_PER_BUF: u64 = VTX_PER_BUF;
    const IDX_PER_BUF: u64 = IDX_PER_BUF;

    fn reset(&mut self) {
        self.scissor = None;
        self.used_indices = 0;
        self.used_vertices = 0;

        self.used_slider_data = 0;
        self.used_slider_grids = 0;
        self.used_grid_cells = 0;
        self.used_line_segments = 0;
    }

    fn dump(&mut self, queue: &wgpu::Queue, cache: &mut Self::Cache) {
        queue.write_buffer(
            &self.vertex_buffer, 
            0, 
            bytemuck::cast_slice(&cache.cpu_vtx)
        );
        queue.write_buffer(
            &self.index_buffer, 
            0, 
            bytemuck::cast_slice(&cache.cpu_idx)
        );
        queue.write_buffer(
            &self.slider_data, 
            0, 
            bytemuck::cast_slice(&cache.slider_data)
        );
        queue.write_buffer(
            &self.slider_grids, 
            0, 
            bytemuck::cast_slice(&cache.slider_grids)
        );
        queue.write_buffer(
            &self.grid_cells, 
            0, 
            bytemuck::cast_slice(&cache.grid_cells)
        );
        queue.write_buffer(
            &self.line_segments, 
            0, 
            bytemuck::cast_slice(&cache.line_segments)
        );
    }

    fn should_write(&self) -> bool {
        self.used_slider_data > 0
    }

    fn create_new_buffer(device: &wgpu::Device, pipeline: WgpuPipeline) -> Self {
        let slider_data = create_buffer::<super::GpuSliderData>(
            device, 
            wgpu::BufferUsages::STORAGE, 
            EXPECTED_SLIDER_COUNT
        );
        let slider_grids = create_buffer::<tataku::GridCell>(
            device, 
            wgpu::BufferUsages::STORAGE, 
            SLIDER_GRID_COUNT
        );
        let grid_cells = create_buffer::<u32>(
            device, 
            wgpu::BufferUsages::STORAGE, 
            GRID_CELL_COUNT
        );
        let line_segments = create_buffer::<tataku::LineSegment>(
            device, 
            wgpu::BufferUsages::STORAGE, 
            LINE_SEGMENT_COUNT
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("slider bind group"),
                layout: &pipeline.get_bind_group_layout(1),
                entries: &[
                    wgpu::BindGroupEntry { 
                        binding: 0, 
                        resource: slider_data.as_entire_binding() 
                    },
                    wgpu::BindGroupEntry { 
                        binding: 1, 
                        resource: slider_grids.as_entire_binding() 
                    },
                    wgpu::BindGroupEntry { 
                        binding: 2, 
                        resource: grid_cells.as_entire_binding() 
                    },
                    wgpu::BindGroupEntry { 
                        binding: 3, 
                        resource: line_segments.as_entire_binding() 
                    },
                ]
            }
        );

        Self {
            scissor: None,

            used_indices: 0,
            used_vertices: 0,
            
            used_slider_data: 0,
            used_slider_grids: 0,
            used_grid_cells: 0,
            used_line_segments: 0,

            vertex_buffer: create_buffer::<super::Vertex>(
                device, 
                wgpu::BufferUsages::VERTEX, 
                VTX_PER_BUF
            ),
            index_buffer: create_buffer::<u32>(
                device, 
                wgpu::BufferUsages::INDEX, 
                IDX_PER_BUF
            ),

            slider_data,
            slider_grids,
            grid_cells,
            line_segments,
            bind_group,
        }
    }
}

/// create slider buffer with COPY_DST usage
fn create_buffer<T>(
    device: &wgpu::Device, 
    t: wgpu::BufferUsages, 
    count: u64
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Slider Buffer"),
        usage: t | wgpu::BufferUsages::COPY_DST,
        size: count * std::mem::size_of::<T>() as u64,
        mapped_at_creation: false,
    })
}


pub(crate) struct CpuBuffer {
    pub cpu_vtx: Vec<super::Vertex>,
    pub cpu_idx: Vec<u32>,

    pub slider_data: Vec<super::GpuSliderData>,
    pub slider_grids: Vec<super::GpuGridCell>,
    pub grid_cells: Vec<u32>,
    pub line_segments: Vec<super::GpuLineSegment>,
}
impl Default for CpuBuffer {
    fn default() -> Self {
        Self {
            cpu_vtx: vec![super::Vertex::default(); VTX_PER_BUF as usize],
            cpu_idx: vec![0; IDX_PER_BUF as usize],

            slider_data: vec![super::GpuSliderData::default(); EXPECTED_SLIDER_COUNT as usize],
            slider_grids: vec![super::GpuGridCell::default(); SLIDER_GRID_COUNT as usize],
            grid_cells: vec![u32::default(); GRID_CELL_COUNT as usize],
            line_segments: vec![super::GpuLineSegment::default(); LINE_SEGMENT_COUNT as usize],
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReserveData<'a> {
    pub vtx: &'a mut [super::Vertex],
    pub idx: &'a mut [u32],

    pub slider_data: &'a mut super::GpuSliderData,
    pub slider_grids: &'a mut [super::GpuGridCell],
    pub grid_cells: &'a mut [u32],
    pub line_segments: &'a mut [super::GpuLineSegment],


    pub idx_offset: u64,
    pub slider_index: u32,
    pub slider_grid_offset: u32,
    pub grid_cell_offset: u32,
    pub line_segment_offset: u32,
}
impl ReserveData<'_> {
    pub fn copy_in(
        &mut self, 
        vtx: &[super::Vertex], 
        idx: &[u32],

        slider_data: super::GpuSliderData,
        slider_grids: &[super::GpuGridCell],
        grid_cells: &[u32],
        line_segments: &[super::GpuLineSegment]
    ) {
        self.vtx.copy_from_slice(vtx);
        self.idx.copy_from_slice(idx);
        
        *self.slider_data = slider_data;
        self.slider_grids.copy_from_slice(slider_grids);
        self.grid_cells.copy_from_slice(grid_cells);
        self.line_segments.copy_from_slice(line_segments);
    }
}
