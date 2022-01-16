use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct Line {
    color: Color,
    p1: Vector2,
    p2: Vector2,
    size: f64,

    depth: f64,
    lifetime: u64,
    spawn_time: u64
}
impl Line {
    pub fn new(p1:Vector2, p2:Vector2, size:f64, depth: f64, color:Color) -> Self {
        Self {
            p1,
            p2,
            size,
            depth,
            color,

            lifetime: 0,
            spawn_time: 0
        }
    }
}
impl Renderable for Line {
    fn get_depth(&self) -> f64 {self.depth}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}

    fn get_spawn_time(&self) -> u64 {self.spawn_time}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}

    fn draw(&mut self, g: &mut GlGraphics, c:Context) {
        graphics::Line::new(self.color.into(), self.size).draw([self.p1.x, self.p1.y, self.p2.x, self.p2.y], &DrawState::default(), c.transform, g);
        // graphics::line_from_to(self.color.into(), self.size, self.p1.into(), self.p2.into(), c.transform, g);
    }
}