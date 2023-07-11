use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub enum Shape {
    /// Square corners
    Square,
    /// Round corners, with resolution per corner.
    Round(f32, u32),
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}


