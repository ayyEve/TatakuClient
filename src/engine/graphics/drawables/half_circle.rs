use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2,
    pub depth: f64,
    pub radius: f64,
    pub left_side: bool,
    draw_state: Option<DrawState>,
}
impl HalfCircle {
    pub fn new(color: Color, pos: Vector2, depth: f64, radius: f64, left_side: bool) -> HalfCircle {
        HalfCircle {
            color,
            pos,
            depth,
            radius,
            left_side,
            draw_state: None,
        }
    }
}
impl Renderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }
    fn get_depth(&self) -> f64 {self.depth}
    fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}

    fn draw(&self, g: &mut GlGraphics, c: Context) {
        self.draw_with_transparency(c, self.color.a, 0.0, g)
    }
}

impl TatakuRenderable for HalfCircle {
    fn draw_with_transparency(&self, c: Context, alpha: f32, _: f32, g: &mut GlGraphics) {
        let start_angle:f64 = if self.left_side {std::f64::consts::PI/2.0} else {std::f64::consts::PI*1.5} as f64;

        graphics::CircleArc::new(
            self.color.alpha(alpha).into(),
            self.radius/2.0,
            start_angle, 
            start_angle + std::f64::consts::PI, 
        ).draw(
            [self.pos.x, self.pos.y, self.radius,self.radius],
            &self.draw_state.unwrap_or(c.draw_state),
            c.transform.trans(-self.radius/2.0, -self.radius/2.0),
            g
        )
    }
}
