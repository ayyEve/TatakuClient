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

    pub color: Color,
    pub depth: f64,
    pub font_size: u32,
    pub text: String,
    pub font: Font,

    lifetime:u64,
    spawn_time:u64
}
impl Text {
    pub fn new(color:Color, depth:f64, pos: Vector2, font_size: u32, text: String, font: Font) -> Text {

        let initial_pos = pos;
        let current_pos = pos;
        let initial_rotation = 0.0;
        let current_rotation = 0.0;
        let initial_color = color;
        let current_color = color;
        let initial_scale = Vector2::one();
        let current_scale = Vector2::one();

        let text_size = measure_text(font.clone(), font_size, &text, Vector2::one());
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
            color,
            depth,
            font_size,
            text,
            font,
            lifetime: 0,
            spawn_time: 0
        }
    }
    pub fn measure_text(&self) -> Vector2 {
        // let mut text_size = Vector2::zero();
        // let mut font = self.font.lock();

        // // let block_char = '█';
        // // let character = font.character(self.font_size, block_char).unwrap();

        // for _ch in self.text.chars() {
        //     let character = font.character(self.font_size, _ch).unwrap();
        //     text_size.x += character.advance_width();
        //     // text_size.y = text_size.y.max(character.offset[1]); //character.advance_height();
        // }
        
        // text_size
        measure_text(self.font.clone(), self.font_size, &self.text, self.current_scale) 
    }
    pub fn center_text(&mut self, rect:Rectangle) {
        let text_size = self.measure_text();
        self.initial_pos = rect.pos + (rect.size - text_size)/2.0; // + Vector2::new(0.0, text_size.y);
        self.current_pos = self.initial_pos;
    }
}
impl Renderable for Text {
    fn get_depth(&self) -> f64 {self.depth}
    fn set_lifetime(&mut self, lifetime:u64) {self.lifetime = lifetime}
    fn get_lifetime(&self) -> u64 {self.lifetime}
    fn set_spawn_time(&mut self, time:u64) {self.spawn_time = time}
    fn get_spawn_time(&self) -> u64 {self.spawn_time}

    fn draw(&mut self, g: &mut GlGraphics, c: Context) {
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
            .trans(-self.origin.x, -self.origin.y + self.measure_text().y)
        ;

        graphics::text(
            self.color.into(),
            self.font_size * self.current_scale.y as u32,
            self.text.as_str(),
            &mut *self.font.lock(),
            transform,
            g
        ).unwrap();
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




fn measure_text(font:Font, font_size: u32, text: &String, scale: Vector2) -> Vector2 {
    let mut text_size = Vector2::zero();
    let mut font = font.lock();

    let block_char = '█';
    let character = font.character(font_size, block_char).unwrap();

    for _ch in text.chars() {
        let character = font.character(font_size, _ch).unwrap();
        text_size.x += character.advance_width();
        text_size.y = text_size.y.max(character.offset[1]); //character.advance_height();
    }
    
    text_size
}

