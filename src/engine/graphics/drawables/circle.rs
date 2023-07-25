use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Circle {
    // current
    pub color: Color,
    pub pos: Vector2,
    pub radius: f32,

    scissor: Scissor,
    blend_mode: BlendMode,

    pub border: Option<Border>,
    pub resolution: u32,
}
impl Circle {
    pub fn new(pos:Vector2, radius:f32, color:Color, border: Option<Border>) -> Circle {

        Circle {
            color,
            pos,
            radius,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,

            border,
            // draw_state: None,
            resolution: 128,
        }
    }
}

impl TatakuRenderable for Circle {
    fn get_name(&self) -> String { "Circle".to_owned() }

    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        let border_alpha = self.border.map(|b|b.color.a).unwrap_or_default();
        self.draw_with_transparency(self.color.a, border_alpha, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, border_alpha: f32, transform: Matrix, g: &mut GraphicsState) {
        let border = self.border.map(|mut b|{ b.color.a = border_alpha; b });

        let transform = transform * Matrix::identity()
            // .scale(self)
            .trans(self.pos)
        ;


        g.draw_circle(self.radius, self.color.alpha(alpha), border, self.resolution, transform, self.scissor, self.blend_mode);

        // graphics::ellipse::Ellipse {
        //     color: self.color.alpha(alpha).into(),
        //     border,
        //     resolution: self.resolution
        // }.draw(
        //     graphics::ellipse::circle(self.pos.x, self.pos.y, self.radius),
        //     &self.draw_state.unwrap_or(c.draw_state),
        //     c.transform,
        //     g
        // );
    }
}

