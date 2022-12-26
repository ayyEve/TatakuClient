mod line;
mod text;
mod font;
mod image;
mod circle;
mod rectangle;
mod animation;
mod sector;
mod half_circle;
mod render_target;
mod skinned_number;

pub use line::*;
pub use font::*;
pub use text::*;
pub use circle::*;
pub use rectangle::*;
pub use animation::*;
pub use sector::*;
pub use half_circle::*;
pub use self::image::*;
pub use render_target::*;
pub use skinned_number::*;


use crate::prelude::*;
pub trait TatakuRenderable:Renderable {
    fn draw_with_transparency(&self, c: Context, alpha: f32, border_alpha: f32, g: &mut GlGraphics);
}