use crate::*;

#[derive(Copy, Clone)]
pub struct Line {
    color: Color,
    vector: Vector2,
    thickness: f32,

    blend_mode: BlendMode,
}
impl Line {
    pub fn new(
        vector: Vector2,
        thickness: f32, 
        color: Color,
    ) -> Self {
        Self {
            vector,
            thickness,
            color,
            blend_mode: BlendMode::AlphaBlending,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for Line {
    fn get_name(&self) -> String { "Line".to_owned() }

    fn get_pipeline(&self) -> GraphicsPipeline { GraphicsPipeline::Standard(self.blend_mode) }
    fn set_pipeline(&mut self, pipeline: GraphicsPipeline) { 
        let GraphicsPipeline::Standard(blend_mode) = pipeline 
        else { return };

        self.blend_mode = blend_mode; 
    }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let color = options.color_with_alpha(self.color);

        g.draw_line(
            self.vector,
            self.thickness, 
            color, 
            transform, 
            self.blend_mode
        );
    }
}
