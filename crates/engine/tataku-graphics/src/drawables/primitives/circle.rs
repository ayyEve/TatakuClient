use crate::*;

#[derive(Copy, Clone)]
#[derive(ChainableInitializer)]
pub struct Circle {
    // current
    pub color: Color,

    pub border: Option<Border>,
    #[chain] pub resolution: u32,
}
impl Circle {
    pub fn new(
        color: Color, 
    ) -> Self {
        Self {
            color,

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

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine,
    ) {
        let color = options.color_with_alpha(self.color);
        let border = self.border.map(|mut b|{ b.color = options.border_color_with_alpha(b.color); b });

        let Some(blend_mode) = options.blend_mode() else { return; };

        g.draw_circle(
            color, 
            border, 
            self.resolution, 
            transform, 
            blend_mode
        );
    }
}
