use crate::*;

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
    SizeChanged(tataku::Vector2),

    /// A file is hovered over the window
    FileHover(PathBuf),

    /// A file was dropped over the window
    FileDrop(PathBuf),

    /// A screenshot has been completed
    ScreenshotComplete(Vec<u8>, [u32; 2], actions::window::ScreenshotInfo),
    
    /// An input event was produced
    Input(input::InputType),

    /// Integrations have been loaded and are sent back to the game for usage
    IntegrationsLoaded(Vec<Box<dyn io::TatakuIntegration>>),

    /// The list of available monitors has been updated
    AvailableMonitors(Vec<String>),

    /// The list of available vsync modes has been updated
    VsyncModes(Vec<tataku::Vsync>),
}
