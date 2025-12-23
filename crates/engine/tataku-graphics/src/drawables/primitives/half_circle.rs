use crate::*;

#[derive(Copy, Clone)]
pub struct HalfCircle {
    pub color: Color,
    pub radius: f32,
    pub left_side: bool,
        blend_mode: BlendMode,
}
impl HalfCircle {
    pub fn new(
        radius: f32, 
        color: Color, 
        left_side: bool
    ) -> Self {
        Self {
            color,
            radius,
            left_side,
            blend_mode: BlendMode::AlphaBlending,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for HalfCircle {
    fn get_name(&self) -> String { "Half Circle".to_owned() }

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
        use std::f32::consts::PI;
        let start_angle = if self.left_side { PI / 2.0 } else { PI * 1.5 };

        g.draw_arc(
            start_angle, 
            start_angle+PI, 
            self.radius, 
            options.color_with_alpha(self.color), 
            None,
            20, 
            transform,
            self.blend_mode
        );
    }
}
