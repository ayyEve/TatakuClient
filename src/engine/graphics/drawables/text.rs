use crate::prelude::*;

#[derive(Clone)]
pub struct Text {
    // current
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f32,
    pub scale: Vector2,

    // pub origin: Vector2,

    text_scale: f32,

    font_size: f32,
    pub text: String,
    pub text_colors: Vec<Color>,
    pub fonts: Vec<Font>,

    scissor: Scissor,
}
impl Text {
    pub fn new(pos: Vector2, font_size: f32, text: String, color:Color, font: Font) -> Text {
        let fonts = vec![font, get_fallback_font()];

        // let text_size = Self::measure_text_internal(&fonts, font_size, &text, Vector2::ONE, 2.0);
        // let origin = text_size / 2.0;

        let base_size = 30.0;
        let text_scale = font_size / base_size;

        Text {
            color,
            pos,
            scale: Vector2::ONE,
            rotation: 0.0,
            text_scale,

            // origin,
            font_size: base_size,
            text,
            fonts,
            text_colors: Vec::new(),
            scissor: None,
        }
    }

    pub fn set_font_size(&mut self, size: f32) {
        let base_size = 30.0;
        self.text_scale = size / base_size;
    }
    
    pub fn measure_text(&self) -> Vector2 {
        Self::measure_text_internal(&self.fonts, self.font_size, &self.text, self.scale * self.text_scale, 2.0) 
    }
    pub fn center_text(&mut self, rect:&Rectangle) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size * rect.scale - text_size) / 2.0;
    }

    pub fn _set_text_colors(&mut self, colors: Vec<Color>) {
        self.text_colors = colors
    }

    
    fn measure_text_internal(fonts: &Vec<Font>, font_size: f32, text: &String, scale: Vector2, line_spacing: f32) -> Vector2 {
        if fonts.len() == 0 { return Vector2::ZERO }

        let mut max_width:f32 = 0.0;
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
                if i.has_character(ch) {
                    current_width += i.get_character(font_size, ch).advance_width;
                    break;
                }
            };
        }

        Vector2::new(
            max_width.max(current_width),
            (font_size + line_spacing) * line_count as f32 - line_spacing
        ) * scale
    }

}
impl TatakuRenderable for Text {
    fn get_name(&self) -> String { format!("Text '{}' with fonts {} and size {}", self.text, self.fonts.iter().map(|f|f.get_name()).collect::<Vec<String>>().join(", "), self.font_size) }
    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s:Scissor) { self.scissor = s }
 
    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, mut transform: Matrix, g: &mut GraphicsState) {
        if self.fonts.len() == 0 { return error!("NO FONT FOR TEXT {}", self.text); }
        let scale = self.scale * self.text_scale;

        transform = transform * Matrix::identity()
            // .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(scale) // scale
            .trans(self.pos) // move to pos
        ;


        let text:Vec<(char, Color)>;
        if self.text_colors.is_empty() {
            text = self.text.chars().map(|c| (c, self.color.alpha(alpha))).collect();
        } else {
            text = self.text.chars().enumerate().map(|(i, c)| (c, self.text_colors[i % self.text_colors.len()].alpha(alpha))).collect();
        }

        let mut x = 0.0;
        let mut y = self.font_size * scale.y;

        // debug!("attempting to draw text");
        for (ch, color) in text {
            if ch == '\n' {
                // move the line down
                y += (self.font_size + 2.0) * self.scale.y;

                // reset x pos
                x = 0.0;
                continue;
            }

            'find_font: for i in self.fonts.iter() {
                // if its not loaded, we want to skip because otherwise we lock the main thread and break everything
                if i.has_char_loaded(ch, self.font_size) {
                    i.draw_character_image(
                        self.font_size, 
                        ch, 
                        [&mut x, &mut y], 
                        scale,
                        color, 
                        self.scissor,
                        transform, 
                        g
                    );
                    break 'find_font;
                }
            }
        }
    }
}
