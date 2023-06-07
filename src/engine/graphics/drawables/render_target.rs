#![allow(unused)]
// use graphics::Viewport;

use crate::prelude::*;

// yoinked form https://github.com/Furball-Engine/Furball.Vixie/blob/master/Furball.Vixie/Graphics/TextureRenderTarget.cs
use std::sync::atomic::{AtomicU32, Ordering};

lazy_static::lazy_static! {
    static ref CURRENT_BOUND:AtomicU32 = AtomicU32::new(0);
}

#[derive(Clone)]
pub struct RenderTarget {
    pub texture: TextureReference,
    pub projection: Matrix,
    pub clear_color: Color,

    pub width: u32,
    pub height: u32,
    pub image: Image,

    _drop_check: Arc<()>
}
impl RenderTarget {
    pub async fn new(width: u32, height: u32, callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) -> TatakuResult<Self> {
        GameWindow::create_render_target((width, height), callback).await
    }

    pub fn new_main_thread(width: u32, height: u32, tex: TextureReference, projection: Matrix, clear_color: Color) -> Self {
        let image = Image::new(
            Vector2::ZERO,
            0.0, 
            tex,
            Vector2::ONE
        );

        Self {
            width,
            height,
            texture: tex,
            projection,
            clear_color,
            image,

            _drop_check: Arc::new(()),
            // image: Image::new()
        }
    }


    pub async fn update(&self, callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) {

    }

}
impl Drop for RenderTarget {
    fn drop(&mut self) {
        if Arc::strong_count(&self._drop_check) == 1 {
            // info!("render target dropped");
            GameWindow::free_render_target(self.texture);
        }
    }
}
