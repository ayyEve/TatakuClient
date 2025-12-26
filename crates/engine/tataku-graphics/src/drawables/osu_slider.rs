use crate::*;

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
    #[cfg(feature="graphics")]
    fn draw(
        &self, 
        options: &DrawOptions,
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let transform = transform * Matrix::identity()
            .scale(self.size);

        let mut slider_data = self.slider_data;
        let alpha = Color::to_f32(self.alpha);

        slider_data.body_color.a = Color::to_u8(Color::to_f32(slider_data.body_color.a) * alpha);
        slider_data.border_color.a = Color::to_u8(Color::to_f32(slider_data.border_color.a) * alpha);

        if !matches!(options.pipeline, None | Some(GraphicsPipeline::Slider))
        {
            error!("expected slider, got {:?}", options.pipeline);
            return;
        }

        g.draw_slider(
            transform,
            slider_data,
            self.slider_grids.clone(),
            self.grid_cells.clone(),
            self.line_segments.clone()
        );
    }
}
