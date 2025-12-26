use crate::*;

#[derive(Copy, Clone)]
#[derive(ChainableInitializer)]
pub struct Rectangle {
    size: Vector2,

    pub color: Color,

    #[chain] pub shape: Shape,
    pub border: Option<Border>,
}
impl Rectangle {
    pub fn new(
        size: Vector2, 
        color: Color, 
    ) -> Self {
        Self {
            size,

            color,
            shape: Shape::Square,

            border: None,
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

#[cfg(feature="graphics")]
impl TatakuRenderable for Rectangle {
    fn get_name(&self) -> String { "Rectangle".to_owned() }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix, 
        g: &mut dyn DrawEngine
    ) {
        let color = options.color_with_alpha(self.color);
        
        let border = self.border.map(|mut b| { 
            b.color = options.border_color_with_alpha(b.color); 
            b 
        });

        let Some(blend_mode) = options.blend_mode() else { return; };

        g.draw_rect(
            self.size,
            border, 
            self.shape, 
            color, 
            transform, 
            blend_mode
        );
    }
}
