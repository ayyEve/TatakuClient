mod shaders;
mod texture;
mod last_drawn;
mod wgpu_engine;
mod buffer_queue;
mod renderable_surface;
mod render_buffer_type;
mod render_buffer_queue_type;

mod prelude {
    pub use crate::shaders::*;
    pub use crate::texture::*;
    pub use crate::last_drawn::*;
    pub use crate::wgpu_engine::*;
    pub use crate::buffer_queue::*;
    pub use crate::renderable_surface::*;
    pub use crate::render_buffer_type::*;
    pub use crate::render_buffer_queue_type::*;

    pub use tracing::*;
}

mod shader_files {
    pub const FLASHLIGHT: &str = include_str!("../shaders/flashlight.wgsl");
    pub const PARTICLES: &str = include_str!("../shaders/particles.wgsl");
    #[cfg(feature="texture_arrays")] 
    pub const SHADER_TEX_ARRAY: &str = include_str!("../shaders/shader_with_tex_array.wgsl");
    #[cfg(not(feature="texture_arrays"))] 
    pub const SHADER: &str = include_str!("../shaders/shader.wgsl");
    pub const SLIDER: &str = include_str!("../shaders/slider.wgsl");
}

use tataku_engine::prelude::*;

pub struct WgpuInit;
#[async_trait]
impl<'window> GraphicsInitializer<'window> for WgpuInit {
    fn name(&self) -> &'static str { "Wgpu Graphics" }

    async fn init(
        &self,
        window: &'window winit::window::Window,
        settings: DisplaySettings
    ) -> TatakuResult<Box<dyn GraphicsEngine + 'window>> {
        Ok(wgpu_engine::WgpuEngine::new(window, &settings).await)
    }
}