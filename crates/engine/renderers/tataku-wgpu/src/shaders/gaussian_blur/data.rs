#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Params {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,

    pub sigma: f32
}
impl Params {
    pub fn new(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
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
