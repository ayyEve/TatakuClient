use crate::prelude::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(serde::Deserialize)]
pub struct Border {
    pub color: Color,
    pub width: f32,
}
impl Border {
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            color, 
            width
        }
    }
}
