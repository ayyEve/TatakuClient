use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2,
    pub radius: f32,
    pub left_side: bool,
    pub scissor: Scissor,
    blend_mode: BlendMode,
    // draw_state: Option<DrawState>,
}
impl HalfCircle {
    pub fn new(pos: Vector2, radius: f32, color: Color, left_side: bool) -> HalfCircle {
        HalfCircle {
            color,
            pos,
            radius,
            left_side,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
            // draw_state: None,
        }
    }
}

impl TatakuRenderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, Vector2::ONE * self.radius) }

    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor (&mut self, s:Scissor) {self.scissor = s}
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(&self, transform: Matrix, g: &mut dyn GraphicsEngine) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut dyn GraphicsEngine) {
        let start_angle = if self.left_side { PI / 2.0 } else { PI * 1.5 };

        g.draw_arc(
            start_angle, 
            start_angle+PI, 
            self.radius, 
            self.color.alpha(alpha), 
            20, 
            transform.trans(self.pos), 
            self.blend_mode
        )

        // TODO: this
        // graphics::CircleArc::new(
        //     self.color.alpha(alpha).into(),
        //     self.radius/2.0,
        //     start_angle, 
        //     start_angle + std::f64::consts::PI, 
        // ).draw(
        //     [self.pos.x, self.pos.y, self.radius,self.radius],
        //     &self.draw_state.unwrap_or(c.draw_state),
        //     c.transform.trans(-self.radius/2.0, -self.radius/2.0),
        //     g
        // )
    }
}
