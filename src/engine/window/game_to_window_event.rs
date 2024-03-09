use image::RgbaImage;
use crate::prelude::*;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot::Sender;

#[allow(unused)]
pub enum Game2WindowEvent {
    ShowCursor,
    HideCursor,
    RequestAttention,
    CloseGame,
    TakeScreenshot(Sender<(Vec<u8>, u32, u32)>),
    LoadImage(LoadImage),

    RefreshMonitors,

    AddEmitter(EmitterRef)
}

pub enum LoadImage {
    Image(RgbaImage, UnboundedSender<TatakuResult<TextureReference>>),
    Font(ActualFont, f32, Option<UnboundedSender<TatakuResult<()>>>),
    FreeTexture(TextureReference),

    CreateRenderTarget((u32, u32), UnboundedSender<TatakuResult<RenderTarget>>, Box<dyn FnOnce(&mut GraphicsState, Matrix) + Send>),
    UpdateRenderTarget(RenderTarget, UnboundedSender<()>, Box<dyn FnOnce(&mut GraphicsState, Matrix) + Send>),
}
