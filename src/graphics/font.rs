use crate::prelude::*;

lazy_static::lazy_static! {
    static ref MAIN_FONT: Font = load_font("main");
    static ref FALLBACK_FONT: Font = load_font("main_fallback");
}

pub fn get_font() -> Font {
    MAIN_FONT.clone()
}

pub fn get_fallback_font() -> Font {
    FALLBACK_FONT.clone()
}

fn load_font(name: &str) -> Font {
    let glyphs = opengl_graphics::GlyphCache::new(format!("resources/fonts/{}.ttf", name), (), opengl_graphics::TextureSettings::new()).unwrap();
    Arc::new(Mutex::new(glyphs))
}