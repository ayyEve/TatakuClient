use tataku_graphics::*;
use tataku_client_common::prelude::*;
use tataku::errors::graphics::GraphicsError;

pub struct DummyGraphicsEngine;
impl RenderingEngine for DummyGraphicsEngine {
    fn is_dummy(&self) -> bool { true }
    fn resize(&mut self, _: [u32; 2]) {}

    fn set_vsync(&mut self, _: Vsync) {}
    fn set_blur(&mut self, _: bool) {}
    fn vsync_modes(&self) -> Vec<Vsync> { Vsync::list() }

    fn create_render_target(
        &mut self, 
        _size: [u32; 2], 
        _clear_color: Color, 
        _do_render: RenderTargetDraw
    ) -> Option<RenderTarget> { None }

    fn update_render_target(
        &mut self, 
        _target: RenderTarget, 
        _do_render: RenderTargetDraw
    ) {}

    fn load_texture_bytes(&mut self, _data: &[u8]) -> TatakuResult<TextureReference> {
        Err(Error::Graphics(GraphicsError::DummyEngine))
    }

    fn load_texture_rgba(&mut self, _data: &[u8], _size: [u32; 2]) -> TatakuResult<TextureReference> {
        Err(Error::Graphics(GraphicsError::DummyEngine))
    }

    fn free_tex(&mut self, _tex: TextureReference, _defer_until_next_draw: bool) {}
    fn screenshot(&mut self, _callback: ScreenshotCallback) {}

    fn begin_render(&mut self) {}
    fn end_render(&mut self) {}
    fn present(&mut self) -> TatakuResult<()> { Ok(()) }

    fn add_emitter(&mut self, _emitter: EmitterReference) {}
    fn update_emitters(&mut self) {}

    fn with_renderer(&mut self, _draw: &dyn Fn(&mut dyn DrawEngine)) {}
}
