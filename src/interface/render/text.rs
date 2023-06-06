// use graphics::{Transformed, CharacterCache};
// use opengl_graphics::GlGraphics;
use crate::prelude::*;

// use std::sync::Arc;
// use parking_lot::Mutex;

// pub fn draw_text<
//     T: DrawableText, 
//     F: FontRender

//     > (
//     text: &T, 
//     font_size: F::Size,
//     font_caches: &Vec<F>, 
//     draw_state: &graphics::DrawState, 
//     transform: graphics::types::Matrix2d, 
//     g: &mut GlGraphics
// ) {

//     if font_caches.len() == 0 {
//         panic!("no fonts!");
//     }

//     let mut x = 0.0;
//     let mut y = 0.0;
//     // let mut font_caches = font_caches.iter().map(|f|f.lock()).collect::<Vec<_>>();

//     for (ch, color) in text.char_colors() {
//         // let mut character = None; //font_caches[0].character(font_size, ch)?;
//         // for font in font_caches.iter_mut() {
//         //     if let Ok(c) = font.character(font_size, ch) {
//         //         character = Some(c);
//         //         if !character.as_ref().unwrap().is_invalid {
//         //             break
//         //         }
//         //     } else {
//         //         panic!("hurr durr")
//         //     }
//         // }
        
//         // if character.as_ref().unwrap().is_invalid {
//         //     // stop font_caches from being borrowed
//         //     character = None;
//         //     // get the error character from the first font
//         //     if let Ok(c) = font_caches[0].character(font_size, ch) {
//         //         character = Some(c);
//         //     }
//         // }
//         // let character = character.unwrap();

//         for i in font_caches {
//             if i.has_character(ch) {
//                 i.draw_character_image(
//                     font_size.clone(), 
//                     ch, 
//                     [&mut x, &mut y], 
//                     color, 
//                     draw_state, 
//                     transform, 
//                     g
//                 );
//                 break;
//             }
//         }

//     }
// }

pub trait DrawableText {
    fn char_colors(&self) -> Vec<(char, Color)>;
}
impl DrawableText for String {
    fn char_colors(&self) -> Vec<(char, Color)> {
        self.chars().map(|c| (c, Color::BLACK)).collect()
    }
}
impl DrawableText for &'static str {
    fn char_colors(&self) -> Vec<(char, Color)> {
        self.chars().map(|c| (c, Color::BLACK)).collect()
    }
}

impl<S:AsRef<str>> DrawableText for (S, Color) {
    fn char_colors(&self) -> Vec<(char, Color)> {
        self.0.as_ref().chars().map(|c| (c, self.1)).collect()
    }
}

impl<S:AsRef<str>> DrawableText for (S, Option<Color>) {
    fn char_colors(&self) -> Vec<(char, Color)> {
        self.0.as_ref().chars().map(|c| (c, self.1.unwrap_or(Color::BLACK))).collect()
    }
}

// can this be better?
impl DrawableText for Vec<(char, Color)> {
    fn char_colors(&self) -> Vec<(char, Color)> {
        self.clone()
    }
}



// pub trait TextRender<F:FontRender>: Renderable + Send + Sync {
//     fn new(color:Color, depth:f64, pos: Vector2, font_size: F::Size, text: String, font: F) -> Self where Self:Sized;
//     fn measure_text(&self) -> Vector2;
//     fn center_text(&mut self, rect:Rectangle);
// }

// pub trait FontRender: Clone where Self:Sized + Send + Sync {
//     type Size: Clone + Send + Sync;
    
//     fn get_name(&self) -> String { "Unnamed".to_owned() }

//     fn size_from_u32(font_size: u32) -> Self::Size;
//     fn size_to_u32(font_size: Self::Size) -> u32;

//     fn get_character(&self, font_size: Self::Size, ch: char) -> FontCharacter;
//     fn has_character(&self, ch: char) -> bool;

//     fn draw_character_image(&self, font_size: Self::Size, ch: char, pos: [&mut f64; 2], color: Color, draw_state: &graphics::DrawState, transform: graphics::types::Matrix2d, graphics: &mut GlGraphics);
// }


// impl FontRender for Arc<Mutex<opengl_graphics::GlyphCache<'static>>> {
//     type Size = u32;

//     fn size_from_u32(font_size: u32) -> Self::Size {
//         font_size
//     }
//     fn size_to_u32(font_size: Self::Size) -> u32 {
//         font_size
//     }

//     fn has_character(&self, ch: char) -> bool {
//         match self.lock().character(0, ch) {
//             Ok(c) => !c.is_invalid,
//             Err(_) => false,
//         }
//     }

//     fn get_character(&self, font_size: Self::Size, ch: char) -> FontCharacter {
//         match self.lock().character(font_size, ch) {
//             Ok(c) => {
//                 FontCharacter {
//                     pos: c.atlas_offset.into(),
//                     size: c.atlas_size.into(),
//                     advance_height: c.advance_height(),
//                     advance_width: c.advance_width(),
//                     top: c.top(),
//                     left: c.left(),
//                 }
//             },
//             Err(_) => FontCharacter::default(),
//         }
//     }

//     fn draw_character_image(&self, font_size: Self::Size, ch: char, [x, y]: [&mut f64; 2], color: Color, draw_state: &graphics::DrawState, transform: graphics::types::Matrix2d, graphics: &mut GlGraphics) {
//         let mut lock = self.lock();
//         let character = lock.character(font_size, ch).unwrap();

//         let mut image = graphics::Image::new_color(color.into());

//         let ch_x = *x + character.left();
//         let ch_y = *y - character.top();
//         image = image.src_rect([
//             character.atlas_offset[0],
//             character.atlas_offset[1],
//             character.atlas_size[0],
//             character.atlas_size[1],
//         ]);
//         image.draw(
//             character.texture,
//             draw_state,
//             transform.trans(ch_x, ch_y),
//             graphics,
//         );

//         *x += character.advance_width();
//         *y += character.advance_height();
//     }
// }

#[derive(Clone, Default)]
pub struct FontCharacter {
    pub pos: Vector2,
    pub size: Vector2,
    pub advance_width: f32,
    pub advance_height: f32,

    pub top: f32,
    pub left: f32,
}