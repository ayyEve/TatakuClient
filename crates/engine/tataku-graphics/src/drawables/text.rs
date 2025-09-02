use crate::prelude::*;

pub struct Text {
    pub layout: parley::Layout<Color>,
    pub blend_mode: GraphicsPipeline,
}
impl Text {
    pub fn new(layout: parley::Layout<Color>) -> Self {
        Self {
            layout,
            blend_mode: GraphicsPipeline::default(),
        }
    }
}
impl TatakuRenderable for Text {
    fn get_blend_mode(&self) -> GraphicsPipeline {
        self.blend_mode
    }

    fn get_name(&self) -> String {
        format!("text")
    }
    fn set_blend_mode(&mut self, blend_mode: GraphicsPipeline) {
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
