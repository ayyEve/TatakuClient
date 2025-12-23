use crate::*;


// this is bad, i dont care
// TODO: care
#[derive(Copy, Clone)]
pub struct Sector {
    pub color: Color,

    pub radius: f32,
    pub start: f32,
    pub end: f32,

    blend_mode: BlendMode,

    pub border: Option<Border>
}
impl Sector {
    pub fn new(
        radius: f32, 
        start: f32, 
        end: f32, 
        color: Color, 
        border: Option<Border>
    ) -> Self {
        Self {
            radius,
            start,
            end,

            color,

            border,
            blend_mode: BlendMode::AlphaBlending,
        }
    }
}

#[cfg(feature="graphics")]
impl TatakuRenderable for Sector {
    fn get_name(&self) -> String { "Sector".to_owned() }

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
        g.draw_arc(
            self.start,
            self.end,
            self.radius,
            options.color_with_alpha(self.color),
            self.border,
            20,
            transform,
            self.blend_mode
        );
    }
}
