use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct SliderDrawable {
    /// bounding size of the slider
    pub size: Vector2,
    /// alpha of whole drawable, mainly used for fade in
    pub alpha: u8,


    /// slider data to be passed onto the shader
    pub slider_data: SliderData,

    /// slider grid data to be passed onto the shader
    pub slider_grids: Vec<GridCell>,

    /// slider grid cells to be passed onto the shader
    pub grid_cells: Vec<u32>,

    /// slider line segments to be passed onto the shader
    pub line_segments: Vec<LineSegment>,
}
impl TatakuRenderable for SliderDrawable {
    fn get_blend_mode(&self) -> GraphicsPipeline { GraphicsPipeline::Slider }
    fn set_blend_mode(&mut self, _blend_mode: GraphicsPipeline) {}

    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        _options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let quad = [
            Vector2::ZERO,
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 0.0),
            Vector2::ONE,
        ];

        let transform = transform * Matrix::identity()
            .scale(self.size)
            .trans(self.slider_data.grid_origin);

        let mut slider_data = self.slider_data;
        let alpha = Color::to_f32(self.alpha);

        slider_data.body_color.a = Color::to_u8(Color::to_f32(slider_data.body_color.a) * alpha);
        slider_data.border_color.a = Color::to_u8(Color::to_f32(slider_data.border_color.a) * alpha);

        g.draw_slider(
            quad,
            transform,
            slider_data,
            self.slider_grids.clone(),
            self.grid_cells.clone(),
            self.line_segments.clone()
        );
    }
}
