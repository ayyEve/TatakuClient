#[macro_use] extern crate log;

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