use crate::prelude::*;
#[derive(Copy, Clone, Default)]
pub struct Transform {
    pub pos: Vector2,
    pub scale: Vector2,
    pub rotation: f32,
    pub origin: Vector2,
}
impl Transform {
    pub fn new(pos: Vector2, scale: Vector2, rotation: f32, origin: Vector2) -> Self {
        Self {
            pos,
            scale,
            rotation,
            origin
        }
    }

    pub fn matrix(&self) -> Matrix {
        Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale) // scale
            .trans(self.pos) // move to pos
    }
}
