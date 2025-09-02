
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Params {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,

    pub size: u32,
}
impl Params {
    pub fn new(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        
        size: u32,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            size,
        }
    }
}
