use crate::prelude::*;

/// generic border object, easier to use than importing graphics::X::Border, and creating it manually
#[derive(Clone, Copy)]
pub struct Border {
    pub color: Color,
    pub radius: f64
}
impl Border {
    pub fn new(color:Color, radius:f64) -> Self {
        Self {
            color, 
            radius
        }
    }
}
impl Into<graphics::rectangle::Border> for Border {
    fn into(self) -> graphics::rectangle::Border {
        graphics::rectangle::Border {
            color: self.color.into(),
            radius: self.radius
        }
    }
}
impl Into<graphics::ellipse::Border> for Border {
    fn into(self) -> graphics::ellipse::Border {
        graphics::ellipse::Border {
            color: self.color.into(),
            radius: self.radius
        }
    }
}
