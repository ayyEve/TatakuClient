use crate::prelude::*;

#[derive(Copy, Clone)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2,
    pub radius: f32,
    pub left_side: bool,
    pub scissor: Scissor,
    blend_mode: Pipeline,
}
impl HalfCircle {
    pub fn new(
        pos: Vector2, 
        radius: f32, 
        color: Color, 
        left_side: bool
    ) -> Self {
        Self {
            color,
            pos,
            radius,
            left_side,
            scissor: None,
            blend_mode: Pipeline::AlphaBlending,
        }
    }
}

impl TatakuRenderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }
    fn get_bounds(&self) -> Bounds { 
        Bounds::new(self.pos, Vector2::ONE * self.radius) 
    }

    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor (&mut self, s: Scissor) { self.scissor = s }
    fn get_blend_mode(&self) -> Pipeline { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: Pipeline) { self.blend_mode = blend_mode }

    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn GraphicsEngine
    ) {
        let start_angle = if self.left_side { PI / 2.0 } else { PI * 1.5 };

        g.draw_arc(
            start_angle, 
            start_angle+PI, 
            self.radius, 
            options.color_with_alpha(self.color), 
            20, 
            transform.trans(self.pos), 
            self.blend_mode
        );
    }
}
