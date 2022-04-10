use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Line {
    color: Color,
    p1: Vector2,
    p2: Vector2,
    size: f64,

    depth: f64,
    context: Option<Context>,
}
impl Line {
    pub fn new(p1:Vector2, p2:Vector2, size:f64, depth: f64, color:Color) -> Self {
        Self {
            p1,
            p2,
            size,
            depth,
            color,
            context: None,
        }
    }
}
impl Renderable for Line {
    fn get_depth(&self) -> f64 {self.depth}
    fn get_context(&self) -> Option<Context> {self.context}
    fn set_context(&mut self, c:Option<Context>) {self.context = c}

    fn draw(&self, g: &mut GlGraphics, c:Context) {
        graphics::Line::new(self.color.into(), self.size).draw([self.p1.x, self.p1.y, self.p2.x, self.p2.y], &DrawState::default(), c.transform, g);
        // graphics::line_from_to(self.color.into(), self.size, self.p1.into(), self.p2.into(), c.transform, g);
    }
}