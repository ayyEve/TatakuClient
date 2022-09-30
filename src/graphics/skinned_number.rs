use crate::prelude::*;

#[derive(Clone)]
pub struct SkinnedNumber {
    // initial
    pub initial_color: Color,
    pub initial_pos: Vector2,
    pub initial_rotation: f64,
    pub initial_scale: Vector2,

    // currentz
    pub current_color: Color,
    pub current_pos: Vector2,
    pub current_rotation: f64,
    pub current_scale: Vector2,

    pub origin: Vector2,

    pub color: Color,
    pub depth: f64,

    number_textures: Vec<Image>,
    symbol_textures: HashMap<char, Image>,

    pub symbol: Option<char>,

    pub number: f64,
    pub floating_precision: usize,
    context: Option<Context>,
}
impl SkinnedNumber {
    pub async fn new<TN: AsRef<str>>(color:Color, depth:f64, pos: Vector2, number: f64, texture_name: TN, symbol: Option<char>, floating_precision: usize) -> TatakuResult<Self> {
        let initial_pos = pos;
        let current_pos = pos;
        let initial_rotation = 0.0;
        let current_rotation = 0.0;
        let initial_color = color;
        let current_color = color;
        let initial_scale = Vector2::one();
        let current_scale = Vector2::one();

        let origin = Vector2::zero();

        let tn = texture_name.as_ref();

        let mut textures =  Vec::new();
        for i in 0..10 {
            let tex = format!("{tn}-{i}");
            let mut tex2 = SkinManager::get_texture(&tex, true).await.ok_or(TatakuError::String(format!("texture does not exist: {}", &tex)))?;
            tex2.origin = origin;
            // tex2.size = tex2.tex_size();
            textures.push(tex2)
        }


        // try to load symbols
        // x, %, ',', .,
        let chars = [
            ('x', "x"),
            ('.', "dot"),
            (',', "comma"),
            ('%', "percent"),
        ];

        let mut symbol_textures = HashMap::new();
        for (c, name) in chars {
            let name = format!("{}-{}", tn, name);
            if let Some(mut tex) = SkinManager::get_texture(name, true).await {
                tex.origin = origin;
                symbol_textures.insert(c, tex);
            }
        }


        Ok(Self {
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
            number,

            number_textures: textures,
            symbol_textures,
            symbol,
            floating_precision,

            context: None
        })
    }

    pub fn number_as_text(&self) -> String {
        let mut s = crate::format_float(self.number, self.floating_precision);

        if self.floating_precision == 0 {
            s = s.split(".").next().unwrap().to_owned();
        }

        if let Some(symb) = self.symbol {
            s.push(symb);
        }

        s
    }

    
    pub fn get_char_tex(&self, c: char) -> Option<&Image> {
        let num = match c {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            _ => return self.symbol_textures.get(&c), //panic!("trying to get non-number char"),
        };
        Some(self.get_num_tex(num))
    }
    pub fn get_num_tex(&self, num: usize) -> &Image {
        if num > 9 {panic!("trying to get tex for num > 9")}
        &self.number_textures[num]
    }
    
    pub fn measure_text(&self) -> Vector2 {
        let s = self.number_as_text();

        let mut width = 0.0;
        let mut max_height:f64 = 0.0;

        for c in s.chars() {
            if let Some(t) = self.get_char_tex(c) {
                let t = t.size() * self.current_scale;
                width += t.x;
                max_height = max_height.max(t.y)
            }
        }
        
        Vector2::new(width, max_height)
    }

    pub fn center_text(&mut self, rect:Rectangle) {
        let text_size = self.measure_text();
        self.initial_pos = rect.current_pos + (rect.size - text_size) / 2.0;
        self.current_pos = self.initial_pos;
    }
}
impl Renderable for SkinnedNumber {
    fn get_depth(&self) -> f64 {self.depth}
    fn get_context(&self) -> Option<Context> {self.context}
    fn set_context(&mut self, c:Option<Context>) {self.context = c}

    fn draw(&self, g: &mut GlGraphics, context: Context) {
        // let size = self.measure_text();

        // from image
        // let pre_rotation = self.current_pos / self.current_scale + self.origin;

        // ignore origin for now, will be pain


        //TODO: cache `s`
        let s = self.number_as_text();
        let mut current_pos = self.current_pos;
        for c in s.chars() {
            if let Some(t) = self.get_char_tex(c) {
                let mut t = t.clone();
                t.current_pos = current_pos;
                t.current_scale = self.current_scale;
                t.current_color = self.current_color;
                current_pos.x += t.size().x;

                t.draw(g, context);
            }
        }

        // let transform = c
        //     .transform
        //     // scale to size
        //     // .scale(self.current_scale.x, self.current_scale.y)

        //     // move to pos
        //     .trans(pre_rotation.x, pre_rotation.y)

        //     // rotate to rotate
        //     .rot_rad(self.current_rotation)
            
        //     // apply origin
        //     .trans(-self.origin.x, -self.origin.y + self.measure_text().y)
        // ;
        
        // ayyeve_piston_ui::render::draw_text(
        //     &(&self.text, self.color), 
        //     (self.font_size as f64 * self.current_scale.y) as u32, 
        //     false, 
        //     &self.fonts, 
        //     &c.draw_state, 
        //     transform, 
        //     g
        // ).unwrap();

        // graphics::text(
        //     self.color.into(),
        //     self.font_size * self.current_scale.y as u32,
        //     self.text.as_str(),
        //     &mut *self.font.lock(),
        //     transform,
        //     g
        // ).unwrap();
    }
}

impl Transformable for SkinnedNumber {
    fn apply_transform(&mut self, transform: &Transformation, val: TransformValueResult) {
        match transform.trans_type {
            TransformType::Position { .. } => {
                let val:Vector2 = val.into();
                self.current_pos = self.initial_pos + val;
            }
            TransformType::Scale { .. } => {
                let val:f64 = val.into();
                self.current_scale = self.initial_scale + val;
            }
            TransformType::Rotation { .. } => {
                let val:f64 = val.into();
                self.current_rotation = self.initial_rotation + val;
            }
            
            // self color
            TransformType::Transparency { .. } => {
                let val:f64 = val.into();
                self.current_color = self.current_color.alpha(val.clamp(0.0, 1.0) as f32);
            }
            TransformType::Color { .. } => {
                let col = val.into();
                self.current_color = col;
            }

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

