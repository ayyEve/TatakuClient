use crate::prelude::Color;

#[derive(serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn is_nonzero(&self) -> bool {
        self.color.a > 0 && self.width > 0.0 
    }
}
