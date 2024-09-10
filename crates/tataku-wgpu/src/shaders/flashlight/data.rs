use tataku_client_common::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlashlightDataInner {
    pub cursor_pos: [f32; 2],
    pub flashlight_radius: f32,
    pub fade_radius: f32,
    pub color: [f32; 4]
}
impl From<FlashlightData> for FlashlightDataInner {
    fn from(value: FlashlightData) -> Self {
        Self {
            cursor_pos:value.cursor_pos.into(),
            flashlight_radius: value.flashlight_radius,
            fade_radius: value.fade_radius,
            color: value.color.into()
        }
    }
}

