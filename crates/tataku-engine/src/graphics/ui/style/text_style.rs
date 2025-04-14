use crate::prelude::*;

#[derive(ChainableInitializer)]
#[derive(Clone)]
pub struct TextStyle {
    #[chain] pub font: Font,
    #[chain] pub font_size: f32,
    #[chain] pub color: Color,
    #[chain] pub line_height: f32,

    #[chain] pub alignment: Alignment,
}
impl TextStyle {
    pub fn measure_text(
        &self, 
        text: &str, 
        scale: Option<Vector2>,
    ) -> Vector2 {
        Text::measure_text_raw(
            &[self.font],
            self.font_size,
            text,
            scale.unwrap_or(Vector2::ONE),
            self.line_height
        )
    }

    /// create and layout some text within the provided bounds
    pub fn create_text(&self, text: String, bounds: Bounds) -> Text {
        let mut text = Text::new(
            Vector2::ZERO,
            self.font_size,
            text,
            self.color,
            self.font
        );
        text.line_height = self.line_height;

        text.pos = self.alignment.resolve(
            &bounds, 
            self.measure_text(&text.text, None), 
            true, 
            true
        );

        text
    }

}

impl Default for TextStyle {
    fn default() -> Self {
        Self { 
            font: Font::Main, 
            font_size: 32.0, 
            color: Color::WHITE, 

            // idk what a sane default for this is
            line_height: 35.0,

            alignment: Alignment::CENTER_LEFT,
        }
    }
}