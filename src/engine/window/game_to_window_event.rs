use image::RgbaImage;
use crate::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

pub enum Game2WindowEvent {
    ShowCursor,
    HideCursor,
    RequestAttention,
    CloseGame,
    TakeScreenshot(Fuze<(Vec<u8>, u32, u32)>),

    LoadImage(LoadImage),

    RefreshMonitors,
}

pub enum LoadImage {
    Image(RgbaImage, UnboundedSender<TatakuResult<TextureReference>>),
    Font(Font, f32, Option<UnboundedSender<TatakuResult<()>>>),
    FreeTexture(TextureReference),

    CreateRenderTarget((u32, u32), UnboundedSender<TatakuResult<RenderTarget>>, RenderPipeline, Box<dyn FnOnce(&mut GraphicsState, Matrix) + Send>),
    UpdateRenderTarget(RenderTarget, UnboundedSender<()>, RenderPipeline, Box<dyn FnOnce(&mut GraphicsState, Matrix) + Send>),
}
