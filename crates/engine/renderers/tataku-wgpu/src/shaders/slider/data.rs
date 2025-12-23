use crate::prelude::*;

/// Vertex buffer layout for sliders
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuSliderData {
    /// Radius of inner slider body
    pub circle_radius: f32,
    /// Width of border around slider body
    pub border_width: f32,

    /// snaking progress as a percentage (0-1)
    pub snake_percentage: f32,

    // slider velocity (neb to describe this properly)
    pub slider_velocity: f32,

    /// Size of the slider in grid units
    pub grid_size: [u32; 2],
    /// Grid cells of this slider. This represents the start index into the
    //// `slider_grids` array, where the length of the slice is the area of the
    // grid, as given by `grid_size`.
    pub grid_index: u32,
    pub _padding: u32,

    /// Colour of the body of slider
    pub body_color: [f32; 4],
    /// Colour of the border of the slider
    pub border_color: [f32; 4],
}
impl From<tataku::SliderData> for GpuSliderData {
    fn from(value: tataku::SliderData) -> Self {
        Self {
            circle_radius: value.circle_radius,
            border_width: value.border_width,
            snake_percentage: value.snake_percentage,
            slider_velocity: value.slider_velocity,
            grid_size: value.grid_size,
            grid_index: value.grid_index,
            _padding: 0,
            body_color: value.body_color.into(),
            border_color: value.border_color.into(),
        }
    }
}


/// Slice into the index buffer representing the grid cell
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuGridCell {
    /// Starting index for slice in `grid_cells` array
    pub index: u32,
    /// Length of slice in `grid_cells` array
    pub length: u32,
}
impl From<tataku::GridCell> for GpuGridCell {
    fn from(value: tataku::GridCell) -> Self {
        Self {
            index: value.index,
            length: value.length,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuLineSegment {
    pub p1: [f32; 2],
    pub p2: [f32; 2],
}
impl From<tataku::LineSegment> for GpuLineSegment {
    fn from(value: tataku::LineSegment) -> Self {
        Self {
            p1: value.p1.into(),
            p2: value.p2.into(),
        }
    }
}
