use crate::prelude::*;

use std::mem::size_of;

pub struct SliderRender {
    // todo: make buffers

    // Radius of inner slider body
    pub circle_radius: f32,
    // Width of border around slider body
    pub border_width: f32,

    // Grids for different sliders. Slices of this array represent an individual slider grid,
    // where each value is a slice into the `grid_cells` array.
    pub slider_grids: Vec<GridCell>,
    // Grid cells for different sliders. Slices of this array represent an individual cell,
    // where each value is an index into the `line_segments` array.
    pub grid_cells: Vec<u32>,
    // Line segments of all sliders in the current render
    pub line_segments: Vec<LineSegment>,
}

/// Vertex buffer layout for sliders
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SliderBuffer {
    pub position: [f32; 2],

    /// Origin position of grid in viewport space
    pub grid_origin: [f32; 2],
    /// Size of the slider in grid units
    pub grid_size: [u32; 2],
    /// Grid cells of this slider. This represents the start index into the
    //// `slider_grids` array, where the length of the slice is the area of the
    // grid, as given by `grid_size`.
    pub grid_index: u32,

    /// Colour of the body of slider
    pub body_color: [f32; 4],
    /// Colour of the border of the slider
    pub border_color: [f32; 4],
}

/// Slice into the index buffer representing the grid cell
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridCell {
    /// Starting index for slice in `grid_cells` array
    pub index: u32,
    /// Length of slice in `grid_cells` array
    pub length: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineSegment {
    pub p1: [f32; 2],
    pub p2: [f32; 2],
}

impl SliderBuffer {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // todo: convert to macro

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // grid origin
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // grid size
                wgpu::VertexAttribute {
                    offset: (2 * size_of::<[f32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32x2,
                },
                // grid index
                wgpu::VertexAttribute {
                    offset: (2 * size_of::<[f32; 2]>() + size_of::<[u32; 2]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
                // body color
                wgpu::VertexAttribute {
                    offset: (2 * size_of::<[f32; 2]>() + size_of::<[u32; 2]>() + size_of::<u32>()) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // border color
                wgpu::VertexAttribute {
                    offset: (2 * size_of::<[f32; 2]>() + size_of::<[u32; 2]>() + size_of::<u32>() + size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }
}
