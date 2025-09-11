use crate::*;

#[derive(Copy, Clone)]
#[derive(ChainableInitializer)]
pub struct Circle {
    // current
    pub color: Color,
    pub pos: Vector2,
    pub radius: f32,
    blend_mode: BlendMode,

    pub border: Option<Border>,
    #[chain] pub resolution: u32,
}
impl Circle {
    pub fn new(
        pos: Vector2, 
        radius: f32, 
        color: Color, 
    ) -> Self {
        Self {
            color,
            pos,
            radius,
            blend_mode: BlendMode::AlphaBlending,

            border: None,
            resolution: 128,
        }
    }
    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }
    pub fn border_maybe(mut self, border: Option<Border>) -> Self {
        self.border = border;
        self
    }
}
impl TatakuRenderable for Circle {
    fn get_name(&self) -> String { "Circle".to_owned() }

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
        g: &mut dyn DrawEngine,
    ) {
        let color = options.color_with_alpha(self.color);
        let border = self.border.map(|mut b|{ b.color = options.border_color_with_alpha(b.color); b });

        let transform = transform * Matrix::identity()
            // .scale(self)
            .trans(self.pos)
        ;

        g.draw_circle(
            self.radius, 
            color, 
            border, 
            self.resolution, 
            transform, 
            self.blend_mode
        );
    }
}
