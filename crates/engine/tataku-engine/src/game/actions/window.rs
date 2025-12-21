use image::RgbaImage;
use crate::*;

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

impl From<WindowAction> for actions::Action {
    fn from(value: WindowAction) -> Self {
        Self::WindowAction(Box::new(value))
    }
}

pub type LoadImageCallback<T> = Box<dyn FnOnce(tataku::Result<T>) + Send + Sync>;
#[derive(Debug2)]
pub enum LoadImage {
    #[debug(skip)] Image(RgbaImage, LoadImageCallback<tataku::TextureReference>),
    FreeTexture {
        tex: tataku::TextureReference, 
        deferred: bool
    },

    #[debug(skip)] CreateRenderTarget((u32, u32), LoadImageCallback<graphics::RenderTarget>, graphics::RenderTargetDraw),
    #[debug(skip)] UpdateRenderTarget(graphics::RenderTarget, LoadImageCallback<()>, graphics::RenderTargetDraw),
}
impl From<LoadImage> for actions::Action {
    fn from(value: LoadImage) -> Self {
        Self::WindowAction(Box::new(WindowAction::LoadImage(Box::new(value))))
    }
}


#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenshotInfo {
    pub upload: bool,
    // pub region: Option<Bounds>,
}
