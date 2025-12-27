mod atlas;
mod shaders;
mod wgpu_engine;
mod wgpu_pipeline;
mod buffer_queues;
mod pipeline_type;
mod projection_matrix;
mod renderable_surface;
mod pipeline_collection;

mod prelude {
    pub(crate) use tracing::*;
    pub(crate) use std::num::NonZeroU64;

    pub(crate) use crate::shaders;
    pub(crate) use crate::wgpu_engine::*;
    pub(crate) use crate::buffer_queues::*;
    pub(crate) use crate::wgpu_pipeline::*;
    pub(crate) use crate::projection_matrix::*;
    pub(crate) use crate::pipeline_collection::*;
    pub(crate) use crate::pipeline_type::PipelineType;

    pub(crate) use tataku_engine as engine;
    pub(crate) use engine::tataku;
    pub(crate) use tataku_graphics as graphics;
}

mod shader_files {
    pub const BOX_BLUR: &str = include_str!("../shaders/box_blur.wgsl");
    pub const GAUSSIAN_BLUR: &str = include_str!("../shaders/gaussian_blur.wgsl");
    pub const FLASHLIGHT: &str = include_str!("../shaders/flashlight.wgsl");
    pub const PARTICLES: &str = include_str!("../shaders/particles.wgsl");
    pub const SHADER: &str = include_str!("../shaders/shader.wgsl");
    pub const SLIDER: &str = include_str!("../shaders/slider.wgsl");
}

use prelude::*;

pub struct WgpuInit;
pub(crate) const FORMAT:wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[cfg(feature="graphics")]
#[engine::async_trait]
impl<'window> engine::window::GraphicsInitializer<'window> for WgpuInit {
    fn name(&self) -> &'static str { "Wgpu Graphics" }

    async fn init(
        &self,
        window: &'window dyn engine::window::RawWindow,
        settings: engine::settings::display::DisplaySettings
    ) -> tataku::Result<Box<dyn tataku_graphics::RenderingEngine + 'window>> {
        Ok(wgpu_engine::WgpuEngine::create(window, &settings).await)
    }
}
