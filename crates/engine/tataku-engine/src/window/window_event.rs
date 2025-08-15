use crate::prelude::*;

#[allow(unused)]
pub enum WindowEvent {
    /// Window received focus
    GotFocus,

    /// Window lost focus
    LostFocus,

    /// Window was minimized
    Minimized,

    /// Window was closed
    Closed,

    /// Window size changed
    SizeChanged(Vector2),

    /// A file is hovered over the window
    FileHover(PathBuf),

    /// A file was dropped over the window
    FileDrop(PathBuf),

    /// A screenshot has been completed
    ScreenshotComplete(Vec<u8>, [u32; 2], ScreenshotInfo),
    
    /// An input event was produced
    Input(InputType),

    /// Integrations have been loaded and are sent back to the game for usage
    IntegrationsLoaded(Vec<Box<dyn TatakuIntegration>>),

    /// The list of available monitors has been updated
    AvailableMonitors(Vec<String>),

    /// The list of available vsync modes has been updated
    VsyncModes(Vec<Vsync>),
}
