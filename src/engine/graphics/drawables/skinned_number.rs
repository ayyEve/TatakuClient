use crate::prelude::*;

#[derive(Clone)]
pub struct SkinnedNumber {
    pub color: Color,
    pub pos: Vector2,
    pub rotation: f32,
    pub scale: Vector2,

    pub origin: Vector2,
    pub spacing_override: Option<f32>,

    number_textures: Vec<Image>,
    symbol_textures: HashMap<char, Image>,

    pub number: f64,
    pub symbol: Option<char>,
    pub floating_precision: usize,
    
    scissor: Scissor,
    blend_mode: BlendMode,
    cache: Arc<RwLock<(f64, String)>>,
}
impl SkinnedNumber {
    pub async fn new<TN: AsRef<str>>(pos: Vector2, number: f64, color:Color, texture_name: TN, symbol: Option<char>, floating_precision: usize) -> TatakuResult<Self> {
        let rotation = 0.0;
        let scale = Vector2::ONE;
        let tn = texture_name.as_ref();

        let mut textures =  Vec::new();
        for i in 0..10 {
            let tex = format!("{tn}-{i}");
            let mut tex2 = SkinManager::get_texture(&tex, true).await.ok_or(TatakuError::String(format!("texture does not exist: {}", &tex)))?;
            tex2.origin = Vector2::ZERO;
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
                tex.origin = Vector2::ZERO;
                symbol_textures.insert(c, tex);
            }
        }


        Ok(Self {
            color,
            pos,
            scale,
            origin: Vector2::ZERO,
            rotation,

            number,

            cache: Arc::new(RwLock::new((number, Self::number_as_text_base(number, floating_precision, &symbol)))),
            number_textures: textures,
            symbol_textures,
            symbol,
            floating_precision,
            spacing_override: None,
            scissor: None,
            blend_mode: BlendMode::AlphaBlending,
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
        let mut max_height:f32 = 0.0;
        let x_spacing = self.spacing_override.unwrap_or_default() * self.scale.x;

        for c in s.chars() {
            if let Some(t) = self.get_char_tex(c) {
                let t = t.size() * self.scale;
                width += t.x + x_spacing;
                max_height = max_height.max(t.y)
            }
        }
        
        Vector2::new(width - x_spacing, max_height)
    }

    pub fn center_text(&mut self, rect:&Bounds) {
        let text_size = self.measure_text();
        self.pos = rect.pos + (rect.size - text_size) / 2.0;
    }

    fn number_as_text_base(num: f64, precision: usize, symbol: &Option<char>) -> String {
        let mut s = format_float(num, precision);

        if precision == 0 {
            s = s.split(".").next().unwrap().to_owned();
        }

        if let Some(symb) = symbol {
            s.push(*symb);
        }

        s
    }

}


impl TatakuRenderable for SkinnedNumber {
    fn get_name(&self) -> String { "Skinned number".to_owned() }
    fn get_bounds(&self) -> Bounds { Bounds::new(self.pos, self.measure_text()) }


    fn get_scissor(&self) -> Scissor { self.scissor }
    fn set_scissor(&mut self, s:Scissor) { self.scissor = s }
    fn get_blend_mode(&self) -> BlendMode { self.blend_mode }
    fn set_blend_mode(&mut self, blend_mode: BlendMode) { self.blend_mode = blend_mode }

    fn draw(&self, transform: Matrix, g: &mut GraphicsState) {
        self.draw_with_transparency(self.color.a, 0.0, transform, g)
    }

    fn draw_with_transparency(&self, alpha: f32, _: f32, mut transform: Matrix, g: &mut GraphicsState) {
        let color = self.color.alpha(alpha);
        let x_spacing = self.spacing_override.unwrap_or_default() * self.scale.x;


        transform = transform * Matrix::identity()
            .trans(-self.origin) // apply origin
            .rot(self.rotation) // rotate
            .scale(self.scale) // scale
            .trans(self.pos) // move to pos
        ;


        //TODO: cache `s`
        let s = self.number_as_text();
        let mut current_pos = Vector2::ZERO;
        for c in s.chars() {
            let Some(mut t) = self.get_char_tex(c).cloned() else { continue }; 
            t.color = color;
            t.set_scissor(self.scissor);
            t.set_blend_mode(self.blend_mode);
            t.draw(transform.trans(current_pos), g);
            current_pos.x += t.size().x * self.scale.x + x_spacing;
        }
        
    }
}
