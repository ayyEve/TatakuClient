use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2,
    pub depth: f32,
    pub radius: f32,
    pub left_side: bool,
    pub scissor: Scissor
    // draw_state: Option<DrawState>,
}
impl HalfCircle {
    pub fn new(color: Color, pos: Vector2, depth: f32, radius: f32, left_side: bool) -> HalfCircle {
        HalfCircle {
            color,
            pos,
            depth,
            radius,
            left_side,
            scissor: None
            // draw_state: None,
        }
    }
}

impl TatakuRenderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }
    fn get_depth(&self) -> f32 {self.depth}
    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor (&mut self, s:Scissor) {self.scissor = s}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        let start_angle = if self.left_side { PI / 2.0 } else { PI * 1.5 };

        g.draw_arc(
            start_angle, 
            start_angle+PI, 
            self.radius, 
            self.depth, 
            self.color.alpha(alpha), 
            20, 
            transform.trans(self.pos), 
            self.scissor
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
