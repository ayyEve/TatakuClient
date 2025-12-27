pub mod mouse;
pub mod keyboard;

use tataku_engine::*;
pub(crate) fn to_size(s: tataku::Vector2) -> winit::dpi::Size {
    winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
        s.x as f64, 
        s.y as f64
    ))
}
