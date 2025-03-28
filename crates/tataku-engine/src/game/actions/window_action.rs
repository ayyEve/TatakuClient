use crate::prelude::*;
use image::RgbaImage;
use tokio::sync::oneshot::Sender as OneshotSender;

#[allow(unused)]
pub enum WindowAction {
    ShowCursor,
    HideCursor,
    RequestAttention,
    CloseGame,
    TakeScreenshot(ScreenshotInfo),
    LoadImage(LoadImage),
    CopyToClipboard(String),

    RefreshMonitors,

    RenderData(Vec<Arc<dyn TatakuRenderable>>),
    SettingsUpdated(DisplaySettings),

    AddEmitter(EmitterReference),
    // MediaControlEvent(souvlaki::MediaControlEvent),
}

impl std::fmt::Debug for WindowAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WindowAction")
    }
}


pub enum LoadImage {
    Image(RgbaImage, OneshotSender<TatakuResult<TextureReference>>),
    Font(ActualFont, f32, Option<OneshotSender<TatakuResult<()>>>),
    FreeTexture(TextureReference),

    CreateRenderTarget((u32, u32), OneshotSender<TatakuResult<RenderTarget>>, RenderTargetDraw),
    UpdateRenderTarget(RenderTarget, OneshotSender<()>, RenderTargetDraw),
}


#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenshotInfo {
    pub upload: bool,
    // pub region: Option<Bounds>,
}

impl From<WindowAction> for TatakuAction {
    fn from(value: WindowAction) -> Self {
        Self::WindowAction(value)
    }
}
