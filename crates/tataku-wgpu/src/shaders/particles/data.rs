use tataku_client_common::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct EmitterInfoInner {
    pub scale_start: f32,
    pub scale_end: f32,

    pub opacity_start: f32,
    pub opacity_end: f32,

    pub rotation_start: f32,
    pub rotation_end: f32,

    _1: f32,
    _2: f32,
}
impl From<EmitterInfo> for EmitterInfoInner {
    fn from(value: EmitterInfo) -> Self {
        Self {
            scale_start: value.scale_start,
            scale_end: value.scale_end,
            opacity_start: value.opacity_start,
            opacity_end: value.opacity_end,
            rotation_start: value.rotation_start,
            rotation_end: value.rotation_end,
            _1: 0.0,
            _2: 0.0,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct RunInfoInner { dt: f32 }
impl RunInfoInner {
    pub fn new(dt: f32) -> Self { Self { dt } }
}
