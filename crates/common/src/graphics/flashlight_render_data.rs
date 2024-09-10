use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct FlashlightData {
    pub cursor_pos: Vector2,
    pub flashlight_radius: f32,
    pub fade_radius: f32,
    pub color: Color
}
