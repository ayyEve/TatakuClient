use crate::prelude::*;

// TODO: make a TextSpan so we dont need a vec of colors here
#[derive(Clone, Debug)]
pub struct Text {
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f32,
    pub scale: Vector2,

    // pub origin: Vector2,

    font_size: f32,
    pub line_height: f32,

    pub text: String,
    pub fonts: Vec<Font>,

    blend_mode: Pipeline,
}
impl Text {
    pub fn new(
        pos: Vector2, 
        font_size: f32, 
        text: impl ToString, 
        color: Color, 
        font: Font
    ) -> Self {
        Self {
            pos,
            color,
            scale: Vector2::ONE,
            rotation: 0.0,

            // origin,
            font_size,
            line_height: font_size + 3.0,
            text: text.to_string(),
            fonts: vec![font, Font::Fallback],
            blend_mode: Pipeline::AlphaBlending,
        }
    }

    /// return the base font_size and the scale it should be scaled by
    pub fn get_font_size_scaled(font_size: f32) -> (f32, f32) {
        let base_size = 30.0;
        let text_scale = font_size / base_size;
        (base_size, text_scale)
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }
    
    pub fn measure_text(&self) -> Vector2 {
        Self::measure_text_raw(
            &self.fonts, 
            self.font_size, 
            &self.text, 
            self.scale, 
            self.line_height
        ) 
    }
    pub fn center_text(&mut self, rect: &Bounds) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size - text_size) / 2.0;
    }
    /// chaining helper
    pub fn centered(mut self, bounds: &Bounds) -> Self {
        self.center_text(bounds);
        self
    }
    
    #[cfg(not(feature = "graphics"))]
    pub fn measure_text_raw(_: &[Font], _: f32, _: &str, _: Vector2, _: f32) -> Vector2 {
        Vector2::ZERO
    }
    #[cfg(feature = "graphics")]
    pub fn measure_text_raw(
        fonts: &[Font], 
        font_size: f32, 
        text: &str, 
        scale: Vector2, 
        line_height: f32
    ) -> Vector2 {
        if fonts.is_empty() { return Vector2::ZERO }

        let (font_size, text_scale) = Self::get_font_size_scaled(font_size);
        let mut max_width: f32 = 0.0;
        let mut current_width = 0.0;
        let mut line_count = 1;

        for ch in text.chars() {
            if ch == '\n' {
                max_width = max_width.max(current_width);
                current_width = 0.0;
                line_count += 1;
                continue;
            }

            for i in fonts {
                let Some(data) = i.get_character(font_size, ch) else { continue };
                current_width += data.advance_width();
                break;
            };
        }

        Vector2::new(
            max_width.max(current_width),
            line_height * line_count as f32
        ) * scale * text_scale
    }

}
impl TatakuRenderable for Text {
    fn get_name(&self) -> String { format!("Text '{}' with fonts {} and size {}", self.text, self.fonts.iter().map(|f| format!("{f:?}")).collect::<Vec<String>>().join(", "), self.font_size) }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, self.measure_text()) }

    fn get_blend_mode(&self) -> Pipeline { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: Pipeline) { self.blend_mode = blend_mode }
 
    #[cfg(not(feature = "graphics"))]
    fn draw(&self, _: &DrawOptions, _: Matrix, _: &mut dyn GraphicsEngine) {}
    
    #[cfg(feature = "graphics")]
    fn draw(
        &self, 
        options: &DrawOptions, 
        mut transform: Matrix, 
        g: &mut dyn GraphicsEngine
    ) {
        if self.fonts.is_empty() { return error!("NO FONT FOR TEXT {}", self.text); }

        let (font_size, text_scale) = Self::get_font_size_scaled(self.font_size);

        let color = options.color_with_alpha(self.color);
        let scale = self.scale * text_scale;

        transform = transform * Matrix::identity()
            // .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(scale) // scale
            .trans(self.pos) // move to pos
        ;

        let mut x = 0.0;
        let mut y = font_size * scale.y;
        for ch in self.text.chars() {
            if ch == '\n' {
                // move the line down
                y += self.line_height * self.scale.y;

                // reset x pos
                x = 0.0;
                continue;
            }

            'find_font: for i in self.fonts.iter() {
                // if its not loaded, we want to skip because otherwise we lock the main thread and break everything
                if !i.has_char_loaded(ch, font_size) { continue }
                i.draw_character_image(
                    font_size, 
                    ch, 
                    [&mut x, &mut y], 
                    scale,
                    color, 
                    self.blend_mode,
                    transform, 
                    g
                );
                break 'find_font;
            }
        }
    }
}
