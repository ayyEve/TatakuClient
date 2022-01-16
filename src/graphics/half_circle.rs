use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct HalfCircle {
    pub color: Color,
    pub pos: Vector2,
    pub depth: f64,
    pub radius: f64,
    pub left_side: bool,

    spawn_time: u64,
    lifetime: u64,
}
impl HalfCircle {
    pub fn new(color: Color, pos: Vector2, depth: f64, radius: f64, left_side: bool) -> HalfCircle {
        HalfCircle {
            color,
            pos,
            depth,
            radius,
            left_side,

            spawn_time:0,
            lifetime:0
        }
    }
}
impl Renderable for HalfCircle {
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
        let start_angle:f64 = if self.left_side {std::f64::consts::PI/2.0} else {std::f64::consts::PI*1.5} as f64;
        
        graphics::circle_arc(
            self.color.into(), 
            self.radius/2.0,
            start_angle, 
            start_angle + std::f64::consts::PI, 
            [self.pos.x, self.pos.y, self.radius,self.radius],
            c.transform.trans(-self.radius/2.0, -self.radius/2.0), 
        g);
    }
}
