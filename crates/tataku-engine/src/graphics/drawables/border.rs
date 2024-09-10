use crate::prelude::Color;


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