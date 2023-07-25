use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub pos: Vector2,
    pub size: Vector2,
    pub scale: Vector2,
}
impl Bounds {
    pub fn new(pos: Vector2, size: Vector2) -> Self {
        Self { 
            pos, 
            size, 
            scale: Vector2::ONE 
        }
    }
    /// check if these bounds contain a point
    pub fn contains(&self, p:Vector2) -> bool {
        p.x > self.pos.x && p.x < self.pos.x + self.size.x && p.y > self.pos.y && p.y < self.pos.y + self.size.y
    }
}
