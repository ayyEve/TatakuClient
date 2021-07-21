mod vector2;
mod benchmark_helper;
mod fps_display;
mod beatmap_manager;
mod volume_control;

pub use vector2::*;
pub use benchmark_helper::*;
pub use fps_display::*;
pub use beatmap_manager::*;
pub use volume_control::*;


pub fn visibility_bg(pos:Vector2, size:Vector2) -> Box<crate::render::Rectangle> {
    let mut color = crate::render::Color::WHITE;
    color.a = 0.6;
    Box::new(crate::render::Rectangle::new(
        color,
        f64::MAX - 10.0,
        pos,
        size,
        None
    ))
}