use crate::*;

#[derive(Clone)]
pub struct SkinnedNumber {
    pub color: Color,
    pub spacing_override: Option<f32>,

    number_textures: Vec<Image>,
    symbol_textures: HashMap<char, Image>,

    pub number: f64,
    pub symbol: Option<char>,
    pub floating_precision: usize,
    
    blend_mode: BlendMode,
    cache: Arc<RwLock<(f64, String)>>,
}
impl SkinnedNumber {
    #[cfg(feature = "graphics")]
    pub fn new<TN: AsRef<str>>(
        number: f64,
        color: Color,
        texture_name: TN,
        symbol: Option<char>,
        floating_precision: usize,
        skin_manager: &mut dyn SkinProvider,

        source: &TextureSource,
        usage: SkinUsage,
    ) -> tataku::Result<Self> {
        let texture_name = texture_name.as_ref();

        let mut number_textures = Vec::new();
        for i in 0..10 {
            let name = format!("{texture_name}-{i}");
            let tex = skin_manager.get_texture(Path::new(&name), source, usage, false)
                .ok_or(Error::String(format!("texture does not exist: {name}")))?;

            number_textures.push(tex);
        }

        // try to load symbols
        let mut symbol_textures = HashMap::new();
        // x, %, ',', .,
        let chars = [
            ('x', "x"),
            ('.', "dot"),
            (',', "comma"),
            ('%', "percent"),
        ];
        for (c, name) in chars {
            let name = format!("{texture_name}-{name}");
            let Some(tex) = skin_manager.get_texture(Path::new(&name), source, usage, false) else { continue };

            symbol_textures.insert(c, tex);
        }

        Ok(Self {
            color,
            number,

            cache: Arc::new(RwLock::new((number, Self::number_as_text_base(number, floating_precision, symbol.as_ref())))),
            number_textures,
            symbol_textures,
            symbol,
            floating_precision,
            spacing_override: None,
            blend_mode: BlendMode::AlphaBlending,
        })
    }

    pub fn number_as_text(&self) -> String {
        let last = self.cache.read();
        if last.0 == self.number { return last.1.clone(); }
        drop(last);


        let s = Self::number_as_text_base(
            self.number,
            self.floating_precision,
            self.symbol.as_ref()
        );
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
        if num > 9 { panic!("trying to get tex for num > 9") }
        &self.number_textures[num]
    }

    pub fn measure_text(&self) -> Vector2 {
        let s = self.number_as_text();

        let mut width = 0.0;
        let mut max_height:f32 = 0.0;
        let x_spacing = self.spacing_override.unwrap_or_default();

        for c in s.chars() {
            if let Some(t) = self.get_char_tex(c) {
                let t = t.size();
                width += t.x + x_spacing;
                max_height = max_height.max(t.y);
            }
        }

        Vector2::new(width - x_spacing, max_height)
    }

    fn number_as_text_base(
        num: f64,
        precision: usize,
        symbol: Option<&char>
    ) -> String {
        let mut s = format_float(&num, precision);

        if precision == 0 {
            s = s.split(".").next().unwrap().to_owned();
        }

        if let Some(symb) = symbol {
            s.push(*symb);
        }

        s
    }

}


#[cfg(feature="graphics")]
impl TatakuRenderable for SkinnedNumber {
    fn get_name(&self) -> String { "Skinned number".to_owned() }

    fn get_pipeline(&self) -> GraphicsPipeline { GraphicsPipeline::Standard(self.blend_mode) }
    fn set_pipeline(&mut self, pipeline: GraphicsPipeline) { 
        let GraphicsPipeline::Standard(blend_mode) = pipeline 
        else { return };

        self.blend_mode = blend_mode; 
    }

    fn draw(
        &self, 
        options: &DrawOptions, 
        transform: Matrix,
        g: &mut dyn DrawEngine
    ) {
        let color = options.color_with_alpha(self.color);
        let x_spacing = self.spacing_override.unwrap_or_default();

        // TODO: cache `s`
        let s = self.number_as_text();
        let mut current_pos = Vector2::ZERO;

        for c in s.chars() {
            let Some(mut t) = self.get_char_tex(c).cloned() else { continue };
            t.color = color;
            // t.set_scissor(self.scissor);
            t.set_pipeline(GraphicsPipeline::Standard(self.blend_mode));

            let transform = transform * Matrix::identity()
                .trans(current_pos);

            t.draw(options, transform, g);
            current_pos.x += t.size().x + x_spacing;
        }
    }
}
