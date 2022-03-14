use crate::prelude::*;

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
    let glyphs = opengl_graphics::GlyphCache::new(format!("resources/fonts/{}", name), (), opengl_graphics::TextureSettings::new()).unwrap();
    Arc::new(Mutex::new(glyphs))
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
