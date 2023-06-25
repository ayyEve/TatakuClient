use crate::prelude::*;

#[derive(Clone)]
pub struct RenderTarget {
    pub texture: TextureReference,
    pub projection: Matrix,
    pub clear_color: Color,

    pub width: u32,
    pub height: u32,
    pub image: Image,

    drop_check: Arc<()>
}
impl RenderTarget {
    pub async fn new(width: u32, height: u32, pipeline: RenderPipeline, callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) -> TatakuResult<Self> {
        GameWindow::create_render_target((width, height), pipeline, callback).await
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

            drop_check: Arc::new(()),
            // image: Image::new()
        }
    }


    // pub async fn update(&self, callback: impl FnOnce(&mut GraphicsState, Matrix) + Send + 'static) {
    // }

}
impl Drop for RenderTarget {
    fn drop(&mut self) {
        if Arc::strong_count(&self.drop_check) == 1 {
            // info!("render target dropped");
            GameWindow::free_texture(self.texture);
        }
    }
}
