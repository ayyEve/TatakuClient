use crate::prelude::*;

#[allow(unused)]
#[derive(Clone, PartialEq, Debug)]
pub enum Window2GameEvent {
    // window events
    GotFocus,
    LostFocus,
    Minimized,
    Closed,

    FileHover(PathBuf),
    FileDrop(PathBuf),

    ScreenshotComplete(Vec<u8>, [u32; 2], ScreenshotInfo),
    

    // keyboard input
    KeyPress(KeyInput),
    KeyRelease(KeyInput),

    // mouse input
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseMove(Vector2),
    MouseScroll(f32),

    // controller input
    ControllerEvent(gilrs::Event, Arc<String>, gilrs::PowerInfo)
}
