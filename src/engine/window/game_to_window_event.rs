use image::RgbaImage;
use crate::prelude::*;
use tokio::sync::oneshot::Sender;
use tokio::sync::mpsc::UnboundedSender;

#[allow(unused)]
pub enum Game2WindowEvent {
    ShowCursor,
    HideCursor,
    RequestAttention,
    CloseGame,
    TakeScreenshot(Sender<(Vec<u8>, u32, u32)>),
    LoadImage(LoadImage),

    RefreshMonitors,

    AddEmitter(EmitterRef),
    RenderData(Vec<Arc<dyn TatakuRenderable>>)
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
