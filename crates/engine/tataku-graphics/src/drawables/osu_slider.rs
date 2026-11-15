use crate::*;

#[derive(Clone, Debug)]
pub struct SliderDrawable {
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
        let mut slider_data = self.slider_data;
        let alpha = options.alpha.unwrap_or(ColorField::MAX);

        slider_data.body_color.a *= alpha;
        slider_data.border_color.a *= alpha;

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
