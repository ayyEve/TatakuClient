use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Line {
    color: Color,
    p1: Vector2,
    p2: Vector2,
    thickness: f32,

    scissor: Scissor,
    blend_mode: BlendMode,
}
impl Line {
    pub fn new(p1:Vector2, p2:Vector2, thickness:f32, color:Color) -> Self {
        Self {
            p1,
            p2,
            thickness,
            color,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
        }
    }
}
impl TatakuRenderable for Line {
    fn get_name(&self) -> String { "Line".to_owned() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.p1, self.p2 - self.p1) }

    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    // fn draw(&self, transform: Matrix, g: &mut dyn GraphicsEngine) {
    //     self.draw_with_transparency(self.color.a, 0.0, transform, g)
    // }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        let color = options.color_with_alpha(self.color);
        let transform = transform * Matrix::identity().trans(self.p1);

        g.draw_line(
            self.p2 - self.p1, 
            self.thickness, 
            color, 
            transform, 
            self.blend_mode
        );

        // graphics::Line::new(
        //     self.color.alpha(alpha).into(), 
        //     self.thickness
        // ).draw(
        //     [self.p1.x, self.p1.y, self.p2.x, self.p2.y], 
        //     &self.draw_state.unwrap_or(c.draw_state), 
        //     c.transform, 
        //     g
        // );
        // graphics::line_from_to(self.color.into(), self.size, self.p1.into(), self.p2.into(), c.transform, g);
    }
}
