use std::ops::Range;

#[derive(Clone, Debug, Default)]
pub struct EmitterVal {
    pub initial: Range<f32>,
    pub range: Range<f32>
}
impl EmitterVal {
    pub fn new(initial: Range<f32>, range: Range<f32>) -> Self {
        Self {
            initial,
            range
        }
    }
    pub fn init_only(initial: Range<f32>) -> Self {
        Self {
            initial,
            range: 0.0..0.0
        }
    }
}


#[derive(Copy, Clone, Debug, Default)]
pub struct EmitterInfo {
    pub scale_start: f32,
    pub scale_end: f32,

    pub opacity_start: f32,
    pub opacity_end: f32,

    pub rotation_start: f32,
    pub rotation_end: f32,

    _1: f32,
    _2: f32,
}
impl EmitterInfo {
    pub fn new(scale: &EmitterVal, opacity: &EmitterVal, rotation: &EmitterVal) -> Self {
        Self {
            scale_start: scale.range.start,
            scale_end: scale.range.end,

            opacity_start: opacity.range.start,
            opacity_end: opacity.range.end,

            rotation_start: rotation.range.start,
            rotation_end: rotation.range.end,

            _1: 0.0,
            _2: 0.0,
        }
    }
}

use crate::prelude::*;
pub trait EmitterReference: Send + Sync {
    fn get_info(&self) -> EmitterInfo;
    fn get_pool(&self) -> Option<&mut Pool<Particle>>;
}
