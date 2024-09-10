use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Particle {
    pub lifetime: f32,
    pub lifetime_max: f32,

    pub position: Vector2,
    pub velocity: Vector2,

    pub scale: f32,
    pub rotation: f32,

    pub color: Color,
    pub image: TextureReference,
}
