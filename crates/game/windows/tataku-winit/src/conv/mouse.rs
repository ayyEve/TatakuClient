use tataku_engine::*;

pub(crate) fn button(btn: winit::event::MouseButton) -> input::MouseButton {
    use winit::event::MouseButton;
    match btn {
        MouseButton::Left => input::MouseButton::Left,
        MouseButton::Right => input::MouseButton::Right,
        MouseButton::Middle => input::MouseButton::Middle,
        MouseButton::Back => input::MouseButton::Back,
        MouseButton::Forward => input::MouseButton::Forward,
        MouseButton::Other(n) => input::MouseButton::Other(n),
    }
}

use winit::event::MouseScrollDelta;
pub(crate) fn scroll(
    scroll: MouseScrollDelta
) -> input::ScrollType {
    match scroll {
        MouseScrollDelta::LineDelta(x, y) => {
            input::ScrollType::Lines([x as i32, y as i32])
        }
        MouseScrollDelta::PixelDelta(p) => {
            input::ScrollType::Pixels(tataku::Vector2::new(
                p.x as f32,
                p.y as f32,
            ))
        }
    }
}
