use crate::prelude::*;

#[derive(Clone)]
pub struct RenderTarget {
    pub projection: Matrix,
    pub clear_color: Color,

    pub width: u32,
    pub height: u32,
    pub image: Image,
}
#[cfg(feature = "graphics")]
impl RenderTarget {
    pub async fn new(
        width: u32, 
        height: u32, 
        callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + Sync + 'static
    ) -> TatakuResult<Self> {
        GameWindow::create_render_target(
            (width, height), 
            callback
        ).await
    }
}
#[cfg(feature = "graphics")]
impl Drop for RenderTarget {
    fn drop(&mut self) {
        if self.image.reference_count() == 1 {
            // trace!("render target dropped");
            GameWindow::free_texture(*self.image.tex);
        }
    }
}
