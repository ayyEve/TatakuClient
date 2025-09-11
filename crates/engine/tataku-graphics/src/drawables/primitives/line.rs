use crate::*;

#[derive(Copy, Clone)]
pub struct Line {
    color: Color,
    p1: Vector2,
    p2: Vector2,
    thickness: f32,

    blend_mode: BlendMode,
}
impl Line {
    pub fn new(
        p1: Vector2, 
        p2: Vector2, 
        thickness: f32, 
        color: Color,
    ) -> Self {
        Self {
            p1,
            p2,
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
        let transform = transform * Matrix::identity().trans(self.p1);

        g.draw_line(
            self.p2 - self.p1, 
            self.thickness, 
            color, 
            transform, 
            self.blend_mode
        );
    }
}
