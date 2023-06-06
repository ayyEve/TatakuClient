mod text;
mod color;
mod render_collection;

pub use text::*;
pub use color::*;
pub use render_collection::*;

#[derive(Clone, Copy)]
pub struct Border {
    pub color: Color,
    pub radius: f32
}
impl Border {
    pub fn new(color:Color, radius:f32) -> Self {
        Self {
            color, 
            radius
        }
    }
}


/// The shape of the rectangle corners
#[derive(Copy, Clone)]
pub enum Shape {
    /// Square corners
    Square,
    /// Round corners, with resolution per corner.
    Round(f32, u32),
    /// Bevel corners
    Bevel(f32),
}