use crate::prelude::*;
use image::RgbaImage;
use tokio::sync::oneshot::Sender as OneshotSender;

#[allow(unused)]
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
    RenderData(#[debug(skip)] Vec<Arc<dyn TatakuRenderable>>),

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

#[derive(Debug2)]
pub enum LoadImage {
    #[debug(skip)] Image(RgbaImage, OneshotSender<TatakuResult<TextureReference>>),
    Font(ActualFont, f32, #[debug(skip)] Option<OneshotSender<TatakuResult<()>>>),
    FreeTexture(TextureReference),

    #[debug(skip)] CreateRenderTarget((u32, u32), OneshotSender<TatakuResult<RenderTarget>>, RenderTargetDraw),
    #[debug(skip)] UpdateRenderTarget(RenderTarget, OneshotSender<()>, RenderTargetDraw),
}


#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenshotInfo {
    pub upload: bool,
    // pub region: Option<Bounds>,
}
