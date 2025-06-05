use crate::prelude::*;
use image::RgbaImage;

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
    LoadImage(LoadImage),

    /// Copy some text to the clipboard
    CopyToClipboard(String),

    /// Refresh available monitors
    RefreshMonitors,

    /// Update the data to render
    RenderData(#[debug(skip)] Vec<Box<dyn TatakuRenderable>>),

    /// Update the display to match the settings
    SettingsUpdated(DisplaySettings),

    /// Add a particle emitter
    AddEmitter(EmitterReference),
}
impl From<WindowAction> for TatakuAction {
    fn from(value: WindowAction) -> Self {
        Self::WindowAction(value)
    }
}

pub type LoadImageCallback<T> = Box<dyn FnOnce(TatakuResult<T>) + Send + Sync>;
#[derive(Debug2)]
pub enum LoadImage {
    #[debug(skip)] Image(RgbaImage, LoadImageCallback<TextureReference>),
    Font(ActualFont, f32, #[debug(skip)] Option<LoadImageCallback<()>>),
    FreeTexture(TextureReference),

    #[debug(skip)] CreateRenderTarget((u32, u32), LoadImageCallback<RenderTarget>, RenderTargetDraw),
    #[debug(skip)] UpdateRenderTarget(RenderTarget, LoadImageCallback<()>, RenderTargetDraw),
}


#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenshotInfo {
    pub upload: bool,
    // pub region: Option<Bounds>,
}
