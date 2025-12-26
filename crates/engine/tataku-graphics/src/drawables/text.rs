use crate::*;

pub struct Text {
    pub layout: Arc<parley::Layout<Color>>,
}
impl Text {
    pub fn new(layout: impl Into<Arc<parley::Layout<Color>>>) -> Self {
        Self {
            layout: layout.into(),
        }
    }
}
impl TatakuRenderable for Text {
    fn get_name(&self) -> String { "text".into() }

    fn draw(
        &self,
        options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        let Some(blend_mode) = options.blend_mode() else { return; };


        g.draw_text(
            transform,
            blend_mode,
            &self.layout
        );
    }
}
