use crate::prelude::*;

#[derive(Clone)]
pub struct Text {
    // current
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f32,
    pub scale: Vector2,

    pub origin: Vector2,

    pub depth: f32,
    pub font_size: f32,
    pub text: String,
    pub text_colors: Vec<Color>,
    pub fonts: Vec<Font>,

    scissor: Scissor,
}
impl Text {
    pub fn new(color:Color, depth:f32, pos: Vector2, font_size: f32, text: String, font: Font) -> Text {
        let fonts = vec![font, get_fallback_font()];

        let rotation = 0.0;
        let scale = Vector2::ONE;

        let text_size = measure_text(&fonts, font_size, &text, Vector2::ONE, 2.0);
        let origin = text_size / 2.0;

        Text {
            color,
            pos,
            scale,
            rotation,

            origin,
            depth,
            font_size,
            text,
            fonts,
            text_colors: Vec::new(),
            scissor: None,
        }
    }
    
    pub fn measure_text(&self) -> Vector2 {
        measure_text(&self.fonts, self.font_size, &self.text, self.scale, 2.0) 
    }
    pub fn center_text(&mut self, rect:&Rectangle) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size - text_size)/2.0; // + Vector2::new(0.0, text_size.y);
    }

    pub fn _set_text_colors(&mut self, colors: Vec<Color>) {
        self.text_colors = colors
    }
}

impl TatakuRenderable for Text {
    fn get_name(&self) -> String { format!("Text '{}' with fonts {} and size {}", self.text, self.fonts.iter().map(|f|f.get_name()).collect::<Vec<String>>().join(", "), self.font_size) }
    fn get_depth(&self) -> f32 { self.depth }
    fn get_scissor(&self) -> Scissor {self.scissor}
    fn set_scissor(&mut self, s:Scissor) {self.scissor = s}

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, transform: Matrix, g: &mut GraphicsState) {
        
        // from image
        let pre_rotation = self.pos / self.scale + self.origin;

        let transform = transform
            // scale to size
            // .scale(self.current_scale.x, self.current_scale.y)

            // move to pos
            .trans(pre_rotation)

            // rotate to rotate
            .rot(self.rotation)
            
            // apply origin
            .trans(-self.origin)
        ;


        let text;
        if self.text_colors.len() > 0 {
            text = self.text.chars().enumerate().map(|(i, c)| (c, self.text_colors[i % self.text_colors.len()].alpha(alpha))).collect::<Vec<(char, Color)>>();
        } else {
            text = self.text.chars().map(|c| (c, self.color.alpha(alpha))).collect::<Vec<(char, Color)>>();
        }
        
        let font_size = self.font_size * self.scale.y as f32;

        draw_text(
            &text, 
            font_size, 
            &self.fonts, 
            2.0,
            self.scissor, 
            transform, 
            g
        );
    }
}



fn measure_text(fonts: &Vec<Font>, font_size: f32, text: &String, _scale: Vector2, line_spacing: f32) -> Vector2 {
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
    )
}


pub fn draw_text<T: DrawableText> (
    text: &T, 
    font_size: f32,
    font_caches: &Vec<Font>, 
    line_spacing: f32,
    scissor: Scissor,
    transform: Matrix, 
    g: &mut GraphicsState
) {
    if font_caches.len() == 0 {
        panic!("no fonts!");
    }

    let mut x = 0.0;
    let mut y = font_size;

    // debug!("attempting to draw text");
    for (ch, color) in text.char_colors() {
        if ch == '\n' {
            // move the line down
            y += font_size + line_spacing;

            // reset x pos
            x = 0.0;
            continue;
        }


        for i in font_caches {
            if i.has_character(ch) {
                i.draw_character_image(
                    font_size.clone(), 
                    ch, 
                    [&mut x, &mut y], 
                    color, 
                    scissor,
                    transform, 
                    g
                );
                break;
            }
        }

    }

    // debug!("done drawing text");
}
