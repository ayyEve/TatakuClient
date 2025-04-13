// // filterSize: 15,
// // iterations: 2,
// const TILE_DIMENSION: u64 = 128;
// const BATCH: [u8; 2] = [4, 4];

// #[repr(C)]
// #[derive(Copy, Clone, Debug, Default)]
// #[derive(bytemuck::Pod, bytemuck::Zeroable)]
// pub struct BlurParams {
//     pub x: f32,
//     pub y: f32,
//     pub width: f32,
//     pub height: f32,

//     pub filter_dim: i32,
//     pub block_dim: u32,
//     pub flip: u32,
// }
// impl BlurParams {
//     pub fn new(
//         x: f32,
//         y: f32,
//         width: f32,
//         height: f32,
//     ) -> Self {
//         // let block_dim =  TILE_DIMENSION - (settings.filterSize - 1);
//         Self {
//             x,
//             y,
//             width,
//             height,
//             filter_dim: 15,
//             block_dim: 128 - (15 - 1),
//             flip: 
//         }
//     }
// }


#[derive(Copy, Clone, Debug, Default)]
pub struct BlurParams {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub sigma: f32
}
impl BlurParams {
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