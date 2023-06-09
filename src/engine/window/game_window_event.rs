use crate::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Window2GameEvent {
    // window events
    GotFocus,
    LostFocus,
    Minimized,
    Closed,

    FileHover(PathBuf),
    FileDrop(PathBuf),
    

    // keyboard input
    KeyPress(Key),
    KeyRelease(Key),
    Text(String),

    // mouse input
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseMove(Vector2),
    MouseScroll(f32),

    // controller input
    ControllerAxis {
        controller_id: u32,
        axis: u8,
        value: f32,
    },
    ControllerPress {
        controller_id: u32,
        button: u8,
    },
    ControllerRelease {
        controller_id: u32,
        button: u8,
    },
}
