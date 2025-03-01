use crate::prelude::*;

#[allow(unused)]
#[derive(Clone, PartialEq, Debug)]
pub enum Window2GameEvent {
    // window events
    GotFocus,
    LostFocus,
    Minimized,
    Closed,

    SizeChanged(Vector2),

    FileHover(PathBuf),
    FileDrop(PathBuf),

    ScreenshotComplete(Vec<u8>, [u32; 2], ScreenshotInfo),
    
    Input(InputType),
}
