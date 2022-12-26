use crate::prelude::*;

#[derive(Clone)]
pub struct SkinnedNumber {
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f64,
    pub scale: Vector2,

    pub origin: Vector2,
    pub depth: f64,

    number_textures: Vec<Image>,
    symbol_textures: HashMap<char, Image>,

    pub symbol: Option<char>,

    pub number: f64,
    pub floating_precision: usize,
    draw_state: Option<DrawState>,

    cache: Arc<parking_lot::RwLock<(f64, String)>>,
}
impl SkinnedNumber {
    pub async fn new<TN: AsRef<str>>(color:Color, depth:f64, pos: Vector2, number: f64, texture_name: TN, symbol: Option<char>, floating_precision: usize) -> TatakuResult<Self> {
        let rotation = 0.0;
        let scale = Vector2::one();

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
            color,
            pos,
            scale,
            rotation,

            origin,
            depth,
            number,

            cache: Arc::new(parking_lot::RwLock::new((number, Self::number_as_text_base(number, floating_precision, &symbol)))),
            number_textures: textures,
            symbol_textures,
            symbol,
            floating_precision,

            draw_state: None,
        })
    }

    pub fn number_as_text(&self) -> String {
        let last = self.cache.read();
        if last.0 == self.number { return last.1.clone(); }
        drop(last);
        

        let s = Self::number_as_text_base(self.number, self.floating_precision, &self.symbol);
        *self.cache.write() = (self.number, s.clone());
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
                let t = t.size() * self.scale;
                width += t.x;
                max_height = max_height.max(t.y)
            }
        }
        
        Vector2::new(width, max_height)
    }

    pub fn center_text(&mut self, rect:Rectangle) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size - text_size) / 2.0;
    }

    fn number_as_text_base(num: f64, precision: usize, symbol: &Option<char>) -> String {
            let mut s = crate::format_float(num, precision);

            if precision == 0 {
                s = s.split(".").next().unwrap().to_owned();
            }

            if let Some(symb) = symbol {
                s.push(*symb);
            }

            s
    }

}
impl Renderable for SkinnedNumber {
    fn get_name(&self) -> String { "Skinned number".to_owned() }
    fn get_depth(&self) -> f64 {self.depth}
    fn get_draw_state(&self) -> Option<DrawState> {self.draw_state}
    fn set_draw_state(&mut self, c:Option<DrawState>) {self.draw_state = c}

    fn draw(&self, g: &mut GlGraphics, c: Context) {
        self.draw_with_transparency(c, self.color.a, 0.0, g)
    }
}


impl TatakuRenderable for SkinnedNumber {
    fn draw_with_transparency(&self, mut context: Context, alpha: f32, _: f32, g: &mut GlGraphics) {

        let color = self.color.alpha(alpha);
        context.draw_state = self.draw_state.unwrap_or(context.draw_state);

        //TODO: cache `s`
        let s = self.number_as_text();
        let mut current_pos = self.pos;
        for c in s.chars() {
            if let Some(t) = self.get_char_tex(c) {
                let mut t = t.clone();
                t.pos = current_pos;
                t.scale = self.scale;
                t.color = color;
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