use crate::prelude::*;

#[derive(Clone)]
pub struct RenderTarget {
    pub projection: Matrix,
    pub clear_color: Color,

    pub width: u32,
    pub height: u32,
    pub image: Image,

    // drop_check: Arc<()>
}
#[cfg(feature = "graphics")]
impl RenderTarget {
    pub async fn new(
        width: u32, 
        height: u32, 
        callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + 'static
    ) -> TatakuResult<Self> {
        GameWindow::create_render_target((width, height), callback).await
    }

    pub fn new_main_thread(width: u32, height: u32, texture: Arc<TextureReference>, projection: Matrix, clear_color: Color) -> Self {
        let image = Image::new(
            Vector2::ZERO,
            texture,
            Vector2::ONE
        );

        Self {
            width,
            height,
            projection,
            clear_color,
            image,

            // drop_check: Arc::new(()),
            // image: Image::new()
        }
    }


    // pub async fn update(&self, callback: impl FnOnce(&mut dyn GraphicsEngine, Matrix) + Send + 'static) {
    // }

}
#[cfg(feature = "graphics")]
impl Drop for RenderTarget {
    fn drop(&mut self) {
        // 2 because we keep a reference
        if self.image.reference_count() == 1 {
            // info!("render target dropped");
            GameWindow::free_texture(*self.image.tex);
        }
        // if Arc::strong_count(&self.drop_check) == 1 {
        //     // info!("render target dropped");
        //     GameWindow::free_texture(*self.texture);
        // }
    }
}
