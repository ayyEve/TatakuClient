use crate::*;
use tataku::{ Vector2, Bounds };

#[derive(ChainableInitializer)]
#[derive(Copy, Clone, Debug, Default)]
pub struct PlayfieldNonsense {
    pub bounds: Bounds,
    #[chain] pub scale: f32,
    #[chain] pub circle_size: Vector2,
    #[chain] pub flip_vertical: bool,
    #[chain] pub is_fullscreen: bool,
}
impl PlayfieldNonsense {
    pub fn new(
        bounds: Bounds, 
        scale: f32, 
        circle_size: Vector2,
        flip_vertical: bool,
    ) -> Self {
        Self {
            bounds,
            scale,
            circle_size,
            flip_vertical,
            is_fullscreen: false,
        }
    }

    pub fn new_simple(bounds: Bounds) -> Self {
        Self {
            bounds,
            scale: 1.0,
            ..Default::default()
        }
    }
}
