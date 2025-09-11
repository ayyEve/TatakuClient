use crate::*;

pub struct Text {
    pub layout: Arc<parley::Layout<Color>>,
    pub blend_mode: BlendMode,
}
impl Text {
    pub fn new(layout: impl Into<Arc<parley::Layout<Color>>>) -> Self {
        Self {
            layout: layout.into(),
            blend_mode: BlendMode::AlphaBlending,
        }
    }
}
impl TatakuRenderable for Text {
    fn get_name(&self) -> String { "text".into() }

    fn get_pipeline(&self) -> GraphicsPipeline { self.blend_mode.into() }
    fn set_pipeline(&mut self, pipeline: GraphicsPipeline) { 
        let GraphicsPipeline::Standard(blend_mode) = pipeline 
        else { return };

        self.blend_mode = blend_mode; 
    }

    fn draw(
        &self,
        _options: &DrawOptions,
        transform: Matrix,
        g: &mut dyn DrawEngine,
    ) {
        g.draw_text(
            transform,
            self.blend_mode,
            &self.layout
        );
    }
}
