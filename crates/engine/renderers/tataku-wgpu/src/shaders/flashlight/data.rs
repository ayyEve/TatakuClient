use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GpuData {
    pub cursor_pos: [f32; 2],
    pub flashlight_radius: f32,
    pub fade_radius: f32,
    pub color: [f32; 4]
}
impl From<tataku::FlashlightData> for GpuData {
    fn from(value: tataku::FlashlightData) -> Self {
        Self {
            cursor_pos:value.center.into(),
            flashlight_radius: value.flashlight_radius,
            fade_radius: value.fade_radius,
            color: value.color.into()
        }
    }
}
