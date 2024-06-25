use image::RgbaImage;
use crate::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

#[allow(unused)]
pub enum Game2WindowEvent {
    ShowCursor,
    HideCursor,
    RequestAttention,
    CloseGame,
    TakeScreenshot(ScreenshotInfo),
    LoadImage(LoadImage),
    CopyToClipboard(String),

    RefreshMonitors,

    AddEmitter(EmitterRef),
    RenderData(Vec<Arc<dyn TatakuRenderable>>),

    SettingsUpdated(DisplaySettings),
    IntegrationsChanged(IntegrationSettings),
}

impl std::fmt::Debug for Game2WindowEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game2WindowEvent")
    }
}


pub enum LoadImage {
    Image(RgbaImage, UnboundedSender<TatakuResult<TextureReference>>),
    Font(ActualFont, f32, Option<UnboundedSender<TatakuResult<()>>>),
    FreeTexture(TextureReference),

    CreateRenderTarget((u32, u32), UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut dyn GraphicsEngine, Matrix) + Send>),
    UpdateRenderTarget(RenderTarget, UnboundedSender<()>, Box<dyn FnOnce(&mut dyn GraphicsEngine, Matrix) + Send>),
}


#[derive(Clone, Default, PartialEq, Debug)]
pub struct ScreenshotInfo {
    pub upload: bool,

    // pub region: Option<Bounds>,
}