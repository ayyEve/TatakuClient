use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2,
    pub depth: f32,
    pub radius: f32,
    pub left_side: bool,
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
            // draw_state: None,
        }
    }
}

impl TatakuRenderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }
    fn get_depth(&self) -> f32 {self.depth}
    // fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    // fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        let start_angle:f32 = if self.left_side {std::f32::consts::PI/2.0} else {std::f32::consts::PI*1.5} as f32;

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
