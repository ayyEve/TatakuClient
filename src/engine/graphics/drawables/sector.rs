use crate::prelude::*;
// this is bad, i dont care

#[derive(Clone, Copy)]
pub struct Sector {
    pub depth: f32,
    pub radius: f32,
    pub start: f32,
    pub end: f32,

    pub pos: Vector2,
    pub scale: Vector2,
    pub color: Color,

    scissor: Scissor,

    pub border: Option<Border>
}
impl Sector {
    pub fn new(pos:Vector2, radius: f32, start:f32, end:f32, color:Color, depth:f32, border: Option<Border>) -> Self {
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
            scissor: None
        }
    }
}

impl TatakuRenderable for Sector {
    fn get_name(&self) -> String { "Sector".to_owned() }
    fn get_depth(&self) -> f32 { self.depth }
    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s: Scissor) { self.scissor = s }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        g.draw_arc(
            self.start,
            self.end,
            self.radius,
            self.depth,
            self.color.alpha(alpha),
            20,
            transform.scale(self.scale).trans(self.pos),
            self.scissor
        )

        //TODO: this!
        // let r = self.scale * self.radius;
        // graphics::CircleArc::new(
        //     self.color.alpha(alpha).into(), 
        //     self.radius / 4.0,
        //     self.start, 
        //     self.end
        // ).draw(
        //     [self.pos.x, self.pos.y, r.x, r.y],
        //     &self.draw_state.unwrap_or(c.draw_state),
        //     c.transform.trans(-self.radius/2.0, -self.radius/2.0), 
        //     g
        // )

    }
}
