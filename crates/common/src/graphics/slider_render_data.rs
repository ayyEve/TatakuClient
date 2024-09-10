use crate::prelude::*;

pub struct SliderRender {
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
#[derive(Copy, Clone, Debug, Default)]
pub struct SliderData {
    /// Radius of inner slider body
    pub circle_radius: f32,
    /// Width of border around slider body
    pub border_width: f32,

    /// snaking progress as a percentage (0-1)
    pub snake_percentage: f32,

    // slider velocity (neb to describe this properly)
    pub slider_velocity: f32,

    /// Origin position of grid in viewport space
    pub grid_origin: Vector2,
    /// Size of the slider in grid units
    pub grid_size: [u32; 2],
    /// Grid cells of this slider. This represents the start index into the
    //// `slider_grids` array, where the length of the slice is the area of the
    // grid, as given by `grid_size`.
    pub grid_index: u32,

    /// Colour of the body of slider
    pub body_color: Color,
    /// Colour of the border of the slider
    pub border_color: Color,
}

/// Slice into the index buffer representing the grid cell
#[derive(Copy, Clone, Debug, Default)]
pub struct GridCell {
    /// Starting index for slice in `grid_cells` array
    pub index: u32,
    /// Length of slice in `grid_cells` array
    pub length: u32,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct LineSegment {
    pub p1: Vector2,
    pub p2: Vector2,
}
