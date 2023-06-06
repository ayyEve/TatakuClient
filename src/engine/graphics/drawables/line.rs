use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Line {
    color: Color,
    p1: Vector2,
    p2: Vector2,
    thickness: f32,

    depth: f32,
    // draw_state: Option<DrawState>,
}
impl Line {
    pub fn new(p1:Vector2, p2:Vector2, thickness:f32, depth: f32, color:Color) -> Self {
        Self {
            p1,
            p2,
            thickness,
            depth,
            color,
            // draw_state: None,
        }
    }
}
impl TatakuRenderable for Line {
    fn get_name(&self) -> String { "Line".to_owned() }
    fn get_depth(&self) -> f32 {self.depth}
    // fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    // fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        

        let transform = transform.trans(self.p1);

        let d = self.p2 - self.p1;
        g.draw_line([0.0, 0.0, d.x, d.y], self.thickness, self.depth, self.color, transform);

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
