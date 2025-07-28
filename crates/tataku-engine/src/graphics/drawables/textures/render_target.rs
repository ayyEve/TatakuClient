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
impl Drop for RenderTarget {
    fn drop(&mut self) {
        if self.image.reference_count() == 1 {
            println!("dropping render target tex");
            GameWindow::free_texture(*self.image.tex);
        }
    }
}
