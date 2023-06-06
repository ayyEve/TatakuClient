use crate::prelude::*;
// use ayyeve_piston_ui::prelude::{ FontRender, TextRender };

lazy_static::lazy_static! {
    static ref MAIN_FONT:Font = load_font("main.ttf");
    static ref FALLBACK_FONT:Font = load_font("main_fallback.ttf");
    static ref FONT_AWESOME:Font = load_font("font_awesome_6_regular.otf");
}

pub fn get_font() -> Font {
    MAIN_FONT.clone()
}

pub fn get_font_awesome() -> Font {
    FONT_AWESOME.clone()
}

pub fn get_fallback_font() -> Font {
    FALLBACK_FONT.clone()
}

fn load_font(name: &str) -> Font {
    Font::load(format!("resources/fonts/{}", name)).expect(&format!("error loading font {name}"))
}

/// list of points for font awesome font
#[repr(u32)]
#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FontAwesome {
    Backward = 0xf04a,
    Play = 0xf04b,
    Pause = 0xf04c,
    Stop = 0xf04d,
    Forward = 0xf04e,

    Backward_Step = 0xf048,
    Forward_Step = 0xf051,

    Circle_Pause = 0xf28b,
    Circle_Play = 0xf144,
    Circle_Stop = 0xf28d,
}
impl FontAwesome {
    pub fn get_char(&self) -> char {
        let c = *self as u32;
        char::from_u32(c).expect(&format!("invalid char id? {}", c))
    }
}


#[derive(Clone)]
pub struct Font {
    pub name: Arc<String>,
    pub font: Arc<fontdue::Font>,
    // if the size is loaded but the char isnt found, dont try to load the font
    pub loaded_sizes: Arc<RwLock<HashSet<u32>>>,
    // pub textures: Arc<RwLock<HashMap<f32, Arc<Texture>>>>,
    pub characters: Arc<RwLock<HashMap<(u32, char), CharData>>>,
}

impl Font {
    pub fn load<P:AsRef<Path>>(path:P) -> Option<Self> {
        let data = Io::read_file(&path).ok()?;
        let name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();

        let font_settings = fontdue::FontSettings::default();
        let font = fontdue::Font::from_bytes(data, font_settings).ok()?;

        Some(Self {
            name: Arc::new(name),
            font: Arc::new(font),
            loaded_sizes: Arc::new(RwLock::new(HashSet::new())),
            // textures:   Arc::new(RwLock::new(HashMap::new())),
            characters: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn load_font_size(&self, font_size: f32) {
        let font_size = FontSize::new(font_size);

        if self.loaded_sizes.read().contains(&font_size.u32()) { return }

        trace!("loading font {} with size {}", self.name, font_size.f32());
        
        // send tex load request to main thread, and wait for it to complete
        if let Err(e) = GameWindow::load_font_data(self.clone(), font_size.f32()) {
            error!("error loading font {}", e);
        }
    }

    pub fn get_char(&self, font_size: f32, c: char) -> Option<CharData> {
        if !self.font.chars().contains_key(&c) { return None }
        let font_size = FontSize::new(font_size);

        let key = (font_size.u32(), c);

        if !self.characters.read().contains_key(&key) {
            // missing, load it
            self.load_font_size(font_size.f32());
        }

        self.characters.read().get(&key).cloned()
    }

    pub fn get_name(&self) -> String { self.name.to_string() }

    pub fn get_character(&self, font_size: f32, ch: char) -> FontCharacter {
        if let Some(c) = self.get_char(font_size, ch) {
            c.into()
        } else {
            Default::default()
        }
    }

    pub fn has_character(&self, ch: char) -> bool {
        self.font.chars().contains_key(&ch)
    }

    pub fn draw_character_image(
        &self, 
        font_size: f32, 
        ch: char, 
        [x, y]: [&mut f32; 2], 
        color: Color, 
        // draw_state: &graphics::DrawState, 
        transform: Matrix, 
        graphics: &mut GraphicsState
    ) {
        let Some(character) = self.get_char(font_size, ch) else { return; };
        
        let ch_x = *x + character.metrics.xmin as f32;
        let ch_y = *y - (character.metrics.height as f32 + character.metrics.ymin as f32); // y = -metrics.bounds.height - metrics.bounds.ymin

        // info!("draw char '{ch}' with data {:?} at {x},{y}", character.metrics);
        graphics.draw_tex(&character.texture, 0.0, color, false, false, transform.trans(Vector2::new(ch_x, ch_y)));

        *x += character.metrics.advance_width;
        // *y += character.metrics.advance_height as f64;
    }

}

#[derive(Clone)]
pub struct CharData {
    pub texture: TextureReference,
    pub metrics: fontdue::Metrics
}
impl Into<FontCharacter> for CharData {
    fn into(self) -> FontCharacter {
        FontCharacter {
            pos: self.texture.uvs.tl.into(),
            size: Vector2::new(self.texture.width as f32, self.texture.height as f32),
            advance_width: self.metrics.advance_width,
            advance_height: self.metrics.advance_height,
            top: 0.0,
            left: 0.0,
        }
    }
}

/// font size helper since f32 isnt hash
pub struct FontSize(f32, u32);
impl FontSize {
    pub fn new(f:f32) -> Self {
        Self(f, (f * 10.0) as u32)
    }
    pub fn u32(&self) -> u32 {
        self.1
    }
    pub fn f32(&self) -> f32 {
        self.0
    }
}
