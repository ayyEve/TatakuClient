#[derive(Copy, Clone, Debug, Default)]
pub struct GaussianBlurParams {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub sigma: f32
}
impl GaussianBlurParams {
    pub fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        sigma: f32,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            sigma
        }
    }
}
