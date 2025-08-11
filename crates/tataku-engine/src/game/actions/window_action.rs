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
    LoadImage(Box<LoadImage>),

    /// Copy some text to the clipboard
    CopyToClipboard(Arc<str>),

    /// Refresh available monitors
    RefreshMonitors,

    /// Update the data to render
    RenderData(#[debug(skip)] Vec<Box<dyn TatakuRenderable>>),

    /// Update the display to match the settings
    SettingsUpdated(DisplaySettings),

    /// Add a particle emitter
    AddEmitter(EmitterReference),

    DumpAtlas,
}
impl Clone for WindowAction {
    fn clone(&self) -> Self {
        match self {
            Self::DumpAtlas => Self::DumpAtlas,
            Self::CloseGame => Self::CloseGame,
            Self::ShowCursor => Self::ShowCursor,
            Self::HideCursor => Self::HideCursor,
            Self::RefreshMonitors => Self::RefreshMonitors,
            Self::RequestAttention => Self::RequestAttention,
            Self::TakeScreenshot(arg0) => Self::TakeScreenshot(arg0.clone()),
            Self::CopyToClipboard(arg0) => Self::CopyToClipboard(arg0.clone()),
            Self::SettingsUpdated(arg0) => Self::SettingsUpdated(arg0.clone()),

            Self::LoadImage(_) => panic!("trying to clone LoadImage"),
            Self::RenderData(_) => panic!("trying to clone RenderData"),
            Self::AddEmitter(_) => panic!("trying to clone AddEmitter"),
        }
    }
}

impl From<WindowAction> for TatakuAction {
    fn from(value: WindowAction) -> Self {
        Self::WindowAction(Box::new(value))
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
