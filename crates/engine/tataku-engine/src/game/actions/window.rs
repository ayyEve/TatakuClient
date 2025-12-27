use image::RgbaImage;
use crate::*;

pub type LoadImageCallback<T> = Box<dyn FnOnce(tataku::Result<T>) + Send + Sync>;

#[derive(Debug2)]
pub enum WindowAction {
    /// Show the system cursor
    ShowCursor,

    /// Hide the system cursor
    HideCursor,

    /// Request the user's attention
    RequestAttention,

    /// Close the game
    CloseGame,

    /// Take a screenshot
    TakeScreenshot(ScreenshotInfo),

    /// Load an image
    #[debug(skip)] LoadTexture(RgbaImage, LoadImageCallback<tataku::TextureReference>),
    
    /// Free an image
    FreeTexture(tataku::TextureReference),

    /// Copy some text to the clipboard
    CopyToClipboard(ArcStr),

    /// Refresh available monitors
    RefreshMonitors,

    /// Update the data to render
    RenderData(#[debug(skip)] Vec<Box<dyn graphics::TatakuRenderable>>),

    /// Update the display to match the settings
    SettingsUpdated(settings::display::DisplaySettings),

    /// Add a particle emitter
    AddEmitter(tataku::EmitterReference),

    DumpAtlas,
}

impl From<WindowAction> for actions::Action {
    fn from(value: WindowAction) -> Self {
        Self::WindowAction(Box::new(value))
    }
}


#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenshotInfo {
    pub upload: bool,
    // pub region: Option<Bounds>,
}
