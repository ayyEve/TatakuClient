use crate::prelude::*;

lazy_static::lazy_static! {
    static ref MAIN_FONT:ActualFont = ActualFont::load("resources/fonts/main.ttf").expect("Error loading main font");
    static ref FALLBACK_FONT:ActualFont = ActualFont::load("resources/fonts/main_fallback.ttf").expect("Error loading fallback font");
    static ref FONT_AWESOME:ActualFont = ActualFont::load("resources/fonts/font_awesome_6_regular.otf").expect("Error loading font awesome");
}

pub fn preload_fonts() {
    for i in [&*MAIN_FONT, &*FALLBACK_FONT, &*FONT_AWESOME] {
        i.load_font_size(30.0, true);
    }
}

#[derive(Clone)]
pub struct ActualFont {
    pub name: Arc<String>,
    pub font: Arc<fontdue::Font>,
    // if the size is loaded but the char isnt found, dont try to load the font
    pub loaded_sizes: Arc<RwLock<HashSet<u32>>>,
    // pub textures: Arc<RwLock<HashMap<f32, Arc<Texture>>>>,
    pub characters: Arc<RwLock<HashMap<(u32, char), CharData>>>,
    
    // if the size is loaded but the char isnt found, dont try to load the font
    queued_for_load: Arc<RwLock<HashSet<u32>>>,
}
impl ActualFont {
    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        let data = Io::read_file(&path).ok()?;
        let name = path.as_ref().file_name().unwrap().to_string_lossy().to_string();
        let font = fontdue::Font::from_bytes(data, Default::default()).ok()?;

        Some(Self {
            name: Arc::new(name),
            font: Arc::new(font),
            loaded_sizes: Arc::new(RwLock::new(HashSet::new())),
            characters: Arc::new(RwLock::new(HashMap::new())),
            queued_for_load: Default::default()
        })
    }

    pub fn get_name(&self) -> String { self.name.to_string() }

    pub fn load_font_size(&self, font_size: f32, wait: bool) {
        let font_size = FontSize::new(font_size);

        // if this font is already loaded, exit
        if self.loaded_sizes.read().contains(&font_size.u32()) { return }

        // if we're not going to wait for this to load
        if wait {
            if self.queued_for_load.read().contains(&font_size.u32()) {
                // found a race condition
                // trying to load a font in waiting mode while it was loaded without wait more previously.
                // if we can wait for it to load, this function shouldnt have been called on the main thread, so we should be able to manually wait for it
                warn!("waiting for font {} to load with size {}", self.name, font_size.f32());
                while !self.loaded_sizes.read().contains(&font_size.u32()) {}
            }
        } else {
            if self.queued_for_load.read().contains(&font_size.u32()) {
                // if we're already waiting to load this font, exit
                return
            } else {
                warn!("Queuing font load {} with size {}", self.name, font_size.f32());
                self.queued_for_load.write().insert(font_size.u32());
            }
        }

        
        // send tex load request to main thread, and wait for it to complete
        if let Err(e) = GameWindow::load_font_data(self.clone(), font_size.f32(), wait) {
            error!("Error loading font {}", e);
        }
    }

    pub fn get_character(&self, font_size: f32, c: char) -> Option<CharData> {
        if !self.has_character(c) { return None }
        let font_size = FontSize::new(font_size);

        let key = (font_size.u32(), c);
        
        if !self.characters.read().contains_key(&key) {
            // missing, load it
            self.load_font_size(font_size.f32(), true);
        }

        self.characters.read().get(&key).cloned()
    }

    pub fn has_character(&self, ch: char) -> bool {
        self.font.chars().contains_key(&ch)
    }

    pub fn has_char_loaded(&self, ch: char, size: f32) -> bool {
        if !self.has_character(ch) { return false }
        
        let font_size = FontSize::new(size);
        let loaded = self.loaded_sizes.read().contains(&font_size.u32());

        // queue loading the font
        if !loaded { self.load_font_size(font_size.f32(), false); }

        loaded
    }

    pub fn draw_character_image(
        &self, 
        font_size: f32, 
        ch: char, 
        [x, y]: [&mut f32; 2], 
        scale: Vector2,
        color: Color, 
        scissor: Scissor,
        blend_mode: BlendMode,
        transform: Matrix, 
        graphics: &mut GraphicsState
    ) {
        let Some(character) = self.get_character(font_size, ch) else { return; };
        
        let ch_x = *x + character.metrics.xmin as f32 * scale.x;
        let ch_y = *y - (character.metrics.height as f32 + character.metrics.ymin as f32) * scale.y;

        // info!("draw char '{ch}' with data {:?} at {x},{y}", character.metrics);
        // dont apply scale to this transform, its already been applied
        graphics.draw_tex(&character.texture, color, false, false, transform.trans(Vector2::new(ch_x, ch_y)), scissor, blend_mode);

        *x += character.metrics.advance_width * scale.x;
        // *y += character.metrics.advance_height as f32;
    }

}


#[derive(Clone, Default)]
pub enum Font {
    #[default]
    Main,
    Fallback,
    FontAwesome,
    // boxed to keep the Font type small (16 vs 48)
    #[allow(unused)]
    Custom(Box<ActualFont>)
}
impl Font {
    pub fn to_iced(&self) -> iced::Font {
        match self {
            Self::Main => iced::Font::with_name("main"),
            Self::Fallback => iced::Font::with_name("fallback"),
            Self::FontAwesome => iced::Font::with_name("font_awesome"),
            Self::Custom(_) => iced::Font::with_name("custom"),
        }
    }

    pub fn from_iced(f: iced::Font) -> Self {
        let iced::font::Family::Name(name) = f.family else { return Self::Main };

        match name {
            "main" => Self::Main,
            "fallback" => Self::Fallback,
            "font_awesome" => Self::FontAwesome,
            "custom" => unimplemented!("custom fonts are not working yet"),

            _ => Self::Main
        }
    }
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Main => "Main".to_owned(),
            Self::Fallback => "Fallback".to_owned(),
            Self::FontAwesome => "FontAwesome".to_owned(),
            Self::Custom(f) => "Custom('".to_owned() + &f.get_name() + "')"
        };
        write!(f, "Font({name})")
    }
}
impl Deref for Font {
    type Target = ActualFont;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Main => &MAIN_FONT,
            Self::Fallback => &FALLBACK_FONT,
            Self::FontAwesome => &FONT_AWESOME,
            Self::Custom(font) => font
        }
    }
}
