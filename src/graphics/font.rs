use crate::prelude::*;
use ayyeve_piston_ui::prelude::{ FontRender, TextRender };
use parking_lot::RwLock;

lazy_static::lazy_static! {
    static ref MAIN_FONT:Font2 = load_font("main.ttf");
    static ref FALLBACK_FONT:Font2 = load_font("main_fallback.ttf");
    static ref FONT_AWESOME:Font2 = load_font("font_awesome_6_regular.otf");
}

pub fn get_font() -> Font2 {
    MAIN_FONT.clone()
}

pub fn get_font_awesome() -> Font2 {
    FONT_AWESOME.clone()
}

pub fn get_fallback_font() -> Font2 {
    FALLBACK_FONT.clone()
}

fn load_font(name: &str) -> Font2 {
    // info!("loading font {}, main thread {}", name, on_main_thread());
    // let mut glyphs = opengl_graphics::GlyphCache::new(format!("resources/fonts/{}", name), (), opengl_graphics::TextureSettings::new()).unwrap();
    // for i in [10, 11, 12, 14, 15, 18, 20, 32] {
    //     for c in (0x20u8..0x7F).map(|ch| ch as char) {
    //         info!("{name} -> loading ('{c}', {i})");
    //         if let Err(e) = glyphs.character(i, c) {
    //             error!("font: {}", e)
    //         }

    //         info!("{name} -> ('{c}', {i}) exists {}", glyphs.opt_character(i, c).is_some())
    //     }
    // }
    // Arc::new(Mutex::new(glyphs))


    let font = Font2::load(format!("resources/fonts/{}", name)).expect(&format!("error loading font {name}"));
    font
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
pub struct Font2 {
    pub font: Arc<fontdue::Font>,
    pub textures: Arc<parking_lot::RwLock<HashMap<FontSize, Arc<Texture>>>>,
    pub characters: Arc<parking_lot::RwLock<HashMap<(FontSize, char), CharData>>>,
}

impl Font2 {
    pub fn load<P:AsRef<Path>>(path:P) -> Option<Self> {
        let data = std::fs::read(path.as_ref()).ok()?;

        let font_settings = fontdue::FontSettings::default();
        let font = fontdue::Font::from_bytes(data, font_settings).ok()?;

        Some(Self {
            font: Arc::new(font),
            textures:   Arc::new(RwLock::new(HashMap::new())),
            characters: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn load_font_size(&self, font_size: FontSize) {

        // send tex load request to main thread, and wait for it to complete
        if let Err(e) = load_font_data(self.clone(), font_size) {
            if let TatakuError::String(s) = &e {
                if s.is_empty() {
                    return;
                }
            }

            error!("error loading font {}", e);
        }
    
    }

    pub fn get_char(&self, font_size: f32, c: char) -> Option<CharData> {
        if !self.font.chars().contains_key(&c) {return None}

        let font_size = FontSize::new(font_size)?;
        let key = (font_size, c);

        if !self.characters.read().contains_key(&key) {
            // missing, load it
            self.load_font_size(font_size);
        }

        self.characters.read().get(&key).cloned()
    }
}

#[derive(Clone)]
pub struct CharData {
    pub texture: Arc<Texture>,
    pub pos: Vector2,
    pub size: Vector2,
    pub metrics: fontdue::Metrics
}

impl Into<ayyeve_piston_ui::prelude::FontCharacter> for CharData {
    fn into(self) -> ayyeve_piston_ui::prelude::FontCharacter {
        ayyeve_piston_ui::prelude::FontCharacter {
            pos: self.pos,
            size: self.size,
            advance_width: self.metrics.advance_width as f64,
            advance_height: self.metrics.advance_height as f64,
            top: 0.0,
            left: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FontSize(pub f32);
impl FontSize {
    pub fn new(size: f32) -> Option<Self> {
        if !size.is_normal() {return None} 
        Some(Self(size))
    }
}
impl Eq for FontSize {}


impl std::hash::Hash for FontSize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.0 * 10.0) as u32).hash(state);
    }
}


impl FontRender for Font2 {
    type Size = FontSize;

    fn size_from_u32(font_size: u32) -> Self::Size {
        FontSize::new(font_size as f32).unwrap()
    }

    fn size_to_u32(font_size: Self::Size) -> u32 {
        font_size.0 as u32
    }

    fn get_character(&self, font_size: Self::Size, ch: char) -> ayyeve_piston_ui::prelude::FontCharacter {
        self.get_char(font_size.0, ch).unwrap().into()
    }

    fn has_character(&self, ch: char) -> bool {
        self.font.chars().contains_key(&ch)
    }

    fn draw_character_image(&self, font_size: Self::Size, ch: char, [x, y]: [&mut f64; 2], color: Color, draw_state: &graphics::DrawState, transform: graphics::types::Matrix2d, graphics: &mut GlGraphics) {
        let character = self.get_char(font_size.0, ch).unwrap();


        
        let ch_x = *x + character.metrics.xmin as f64;
        let ch_y = *y - (character.metrics.height as f64 + character.metrics.ymin as f64); // y = -metrics.bounds.height - metrics.bounds.ymin
        
        let mut image = graphics::Image::new_color(color.into());
        image = image.src_rect([
            character.pos.x,
            character.pos.y,
            character.size.x,
            character.size.y,
        ]);
        image.draw(
            character.texture.as_ref(),
            draw_state,
            transform.trans(ch_x, ch_y),
            graphics,
        );

        *x += character.metrics.advance_width as f64;
        *y += character.metrics.advance_height as f64;
    }

}

impl TextRender<Font2> for Text {
    fn new(color:Color, depth:f64, pos: Vector2, font_size: <Font2 as FontRender>::Size, text: String, font: Font2) -> Self where Self:Sized {
        Text::new(color, depth, pos, font_size.0 as u32, text, font)
    }

    fn measure_text(&self) -> Vector2 {
        Text::measure_text(&self)
    }

    fn center_text(&mut self, rect:ayyeve_piston_ui::prelude::Rectangle) {
        Text::center_text(self, rect.into())
    }
}