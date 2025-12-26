use crate::*;

#[derive(Copy, Clone)]
pub struct HalfCircle {
    pub color: Color,
    pub left_side: bool,
}
impl HalfCircle {
    pub fn new(
        color: Color, 
        left_side: bool
    ) -> Self {
        Self {
            color,
            left_side,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }

    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn DrawEngine
    ) {
        use std::f32::consts::PI;
        let start_angle = if self.left_side { PI / 2.0 } else { PI * 1.5 };

        let Some(blend_mode) = options.blend_mode() else { return; };

        g.draw_arc(
            start_angle, 
            start_angle+PI, 
            options.color_with_alpha(self.color), 
            None,
            20, 
            transform,
            blend_mode
        );
    }
}
