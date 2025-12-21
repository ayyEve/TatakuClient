mod shaders;
mod texture;
mod last_drawn;
mod wgpu_engine;
mod buffer_queue;
mod renderable_surface;
mod render_buffer_type;
mod render_buffer_queue_type;

mod prelude {
    pub(crate) use tracing::*;
    pub(crate) use std::num::NonZeroU64;
    pub(crate) use std::num::NonZeroU32;

    pub(crate) use crate::shaders;
    pub(crate) use crate::wgpu_engine::*;
    pub(crate) use crate::buffer_queue::*;
    pub(crate) use crate::last_drawn::PipelineType;
    pub(crate) use crate::render_buffer_type::RenderBufferType;
    pub(crate) use crate::render_buffer_queue_type::RenderBufferQueueType;

    pub(crate) use tataku_engine as engine;
    pub(crate) use engine::tataku;
    pub(crate) use tataku_graphics as graphics;
}

mod shader_files {
    pub const BOX_BLUR: &str = include_str!("../shaders/box_blur.wgsl");
    pub const GAUSSIAN_BLUR: &str = include_str!("../shaders/gaussian_blur.wgsl");
    pub const FLASHLIGHT: &str = include_str!("../shaders/flashlight.wgsl");
    pub const PARTICLES: &str = include_str!("../shaders/particles.wgsl");
    #[cfg(feature="texture_arrays")]
    pub const SHADER_TEX_ARRAY: &str = include_str!("../shaders/shader_with_tex_array.wgsl");
    #[cfg(not(feature="texture_arrays"))]
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
        window: &'window winit::window::Window,
        settings: engine::settings::display::DisplaySettings
    ) -> tataku::Result<Box<dyn tataku_graphics::RenderingEngine + 'window>> {
        Ok(wgpu_engine::WgpuEngine::create(window, &settings).await)
    }
}
