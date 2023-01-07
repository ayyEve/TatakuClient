use crate::prelude::*;
// this is bad, i dont care

#[derive(Clone, Copy)]
pub struct Sector {
    pub depth: f64,
    pub radius: f64,
    pub start: f64,
    pub end: f64,

    pub pos: Vector2,
    pub scale: Vector2,
    pub color: Color,

    draw_state: Option<DrawState>,

    pub border: Option<Border>
}
impl Sector {
    pub fn new(pos:Vector2, radius: f64, start:f64, end:f64, color:Color, depth:f64, border: Option<Border>) -> Self {
        let scale = Vector2::ONE;

        Self {
            depth,
            radius,
            start,
            end,

            color,
            pos,
            scale,

            border,
            draw_state: None,
        }
    }
}
impl Renderable for Sector {
    fn get_name(&self) -> String { "Sector".to_owned() }
    fn get_depth(&self) -> f64 {self.depth}
    fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}

    fn draw(&self, g: &mut GlGraphics, c: Context) {
        self.draw_with_transparency(c, self.color.a, 0.0, g)
    }
}

impl TatakuRenderable for Sector {
    fn draw_with_transparency(&self, c: Context, alpha: f32, _: f32, g: &mut GlGraphics) {
        let r = self.scale * self.radius;

        graphics::CircleArc::new(
            self.color.alpha(alpha).into(), 
            self.radius / 4.0,
            self.start, 
            self.end
        ).draw(
            [self.pos.x, self.pos.y, r.x, r.y],
            &self.draw_state.unwrap_or(c.draw_state),
            c.transform.trans(-self.radius/2.0, -self.radius/2.0), 
            g
        )

    }
}