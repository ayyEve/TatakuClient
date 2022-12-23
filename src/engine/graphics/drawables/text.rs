use crate::prelude::*;

#[derive(Clone)]
pub struct Text {
    // initial
    pub initial_color: Color,
    pub initial_pos: Vector2,
    pub initial_rotation: f64,
    pub initial_scale: Vector2,

    // current
    pub current_color: Color,
    pub current_pos: Vector2,
    pub current_rotation: f64,
    pub current_scale: Vector2,

    pub origin: Vector2,

    pub depth: f64,
    pub font_size: FontSize,
    pub text: String,
    pub text_colors: Vec<Color>,
    pub fonts: Vec<Font2>,

    context: Option<Context>,
}
impl Text {
    pub fn new(color:Color, depth:f64, pos: Vector2, font_size: u32, text: String, font: Font2) -> Text {
        let font_size = FontSize::new(font_size as f32).unwrap();
        let fonts = vec![font, get_fallback_font()];

        let initial_pos = pos;
        let current_pos = pos;
        let initial_rotation = 0.0;
        let current_rotation = 0.0;
        let initial_color = color;
        let current_color = color;
        let initial_scale = Vector2::one();
        let current_scale = Vector2::one();

        let text_size = measure_text(&fonts, font_size, &text, Vector2::one(), 2.0);
        let origin = text_size / 2.0;

        Text {
            initial_color,
            current_color,
            initial_pos,
            current_pos,
            initial_scale,
            current_scale,
            initial_rotation,
            current_rotation,

            origin,
            depth,
            font_size,
            text,
            fonts,
            text_colors: Vec::new(),
            context: None,
        }
    }
    
    pub fn measure_text(&self) -> Vector2 {
        measure_text(&self.fonts, self.font_size, &self.text, self.current_scale, 2.0) 
    }
    pub fn center_text(&mut self, rect:&Rectangle) {
        let text_size = self.measure_text();
        self.initial_pos = rect.current_pos + (rect.size - text_size)/2.0; // + Vector2::new(0.0, text_size.y);
        self.current_pos = self.initial_pos;
    }

    pub fn _set_text_colors(&mut self, colors: Vec<Color>) {
        self.text_colors = colors
    }
}
impl Renderable for Text {
    fn get_name(&self) -> String { format!("Text '{}' with fonts {} and size {}", self.text, self.fonts.iter().map(|f|f.get_name()).collect::<Vec<String>>().join(", "), self.font_size.0) }
    fn get_depth(&self) -> f64 { self.depth }
    fn get_context(&self) -> Option<Context> { self.context }
    fn set_context(&mut self, c:Option<Context>) { self.context = c }

    fn draw(&self, g: &mut GlGraphics, c: Context) {
        // from image
        let pre_rotation = self.current_pos / self.current_scale + self.origin;

        let transform = c
            .transform
            // scale to size
            // .scale(self.current_scale.x, self.current_scale.y)

            // move to pos
            .trans(pre_rotation.x, pre_rotation.y)

            // rotate to rotate
            .rot_rad(self.current_rotation)
            
            // apply origin
            .trans(-self.origin.x, -self.origin.y)
        ;


        let text;
        if self.text_colors.len() > 0 {
            text = self.text.chars().enumerate().map(|(i, c)| (c, self.text_colors[i % self.text_colors.len()].alpha(self.current_color.a))).collect::<Vec<(char, Color)>>();
        } else {
            text = self.text.chars().map(|c| (c, self.current_color)).collect::<Vec<(char, Color)>>();
        }
        
        let mut font_size = self.font_size.clone();
        font_size.0 *= self.current_scale.y as f32;

        draw_text(
            &text, 
            font_size, 
            &self.fonts, 
            2.0,
            &c.draw_state, 
            transform, 
            g
        );
    }
}

impl Transformable for Text {
    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                self.current_pos = self.initial_pos + val;
            },
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_scale = self.initial_scale + val;
            },
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.current_rotation = self.initial_rotation + val;
            }
            
            // self color
            TransformType::Transparency { .. } => {
                let val:f64 = val.into();
                self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
            },
            TransformType::Color { .. } => {
                let col = val.into();
                self.current_color = col;
            },

            // border
            // TransformType::BorderTransparency { .. } => if let Some(border) = self.border.as_mut() {
            //     // this is a circle, it doesnt rotate
            //     let val:f64 = val.into();
            //     border.color = border.color.alpha(val.clamp(0.0, 1.0) as f32);
            // },
            // TransformType::BorderSize { .. } => if let Some(border) = self.border.as_mut() {
            //     // this is a circle, it doesnt rotate
            //     border.radius = val.into();
            // },
            // TransformType::BorderColor { .. } => if let Some(border) = self.border.as_mut() {
            //     let val:Color = val.into();
            //     border.color = val
            // },

            TransformType::None => {},
            _ => {}
        }
    }
    
    fn visible(&self) -> bool {
        self.current_scale.x != 0.0 && self.current_scale.y != 0.0
    }
}



fn measure_text(fonts: &Vec<Font2>, font_size: <Font2 as FontRender>::Size, text: &String, _scale: Vector2, line_spacing: f64) -> Vector2 {
    if fonts.len() == 0 { return Vector2::zero() }

    let mut max_width:f64 = 0.0;
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
        (font_size.0 as f64 + line_spacing) * line_count as f64 - line_spacing
    )
}


pub fn draw_text<T: ayyeve_piston_ui::prelude::DrawableText> (
    text: &T, 
    font_size: FontSize,
    font_caches: &Vec<Font2>, 
    line_spacing: f64,
    draw_state: &graphics::DrawState, 
    transform: graphics::types::Matrix2d, 
    g: &mut GlGraphics
) {
    if font_caches.len() == 0 {
        panic!("no fonts!");
    }

    let mut x = 0.0;
    let mut y = font_size.0 as f64;

    // debug!("attempting to draw text");
    for (ch, color) in text.char_colors() {
        if ch == '\n' {
            // move the line down
            y += font_size.0 as f64 + line_spacing;

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
                    draw_state, 
                    transform, 
                    g
                );
                break;
            }
        }

    }

    // debug!("done drawing text");
}
