use crate::prelude::*;

pub struct DummyGraphicsEngine;

impl GraphicsEngine for DummyGraphicsEngine {
    fn is_dummy(&self) -> bool { true }
    fn resize(&mut self, _: [u32; 2]) {}

    fn set_vsync(&mut self, _: Vsync) {}

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
        Err(TatakuError::Graphics(GraphicsError::DummyEngine))
    }

    fn load_texture_rgba(&mut self, _data: &[u8], _size: [u32; 2]) -> TatakuResult<TextureReference> {
        Err(TatakuError::Graphics(GraphicsError::DummyEngine))
    }

    fn free_tex(&mut self, _tex: TextureReference) {}

    fn screenshot(&mut self, _callback: ScreenshotCallback) {}

    fn begin_render(&mut self) {}
    fn end_render(&mut self) {}
    fn present(&mut self) -> TatakuResult<()> { Ok(()) }

    fn push_scissor(&mut self, _scissor: [f32; 4]) {}

    fn pop_scissor(&mut self) {}

    fn draw_arc(&mut self, _start: f32, _end: f32, _radius: f32, _color: Color, _resolution: u32, _transform: Matrix, _blend_mode: BlendMode) {}

    fn draw_circle(&mut self, _radius: f32, _color: Color, _border: Option<Border>, _resolution: u32, _transform: Matrix, _blend_mode: BlendMode) {}

    fn draw_line(&mut self, _p: Vector2, _thickness: f32, _color: Color, _transform: Matrix, _blend_mode: BlendMode) {}

    fn draw_rect(&mut self, _rect: [f32; 4], _border: Option<Border>, _shape: Shape, _color: Color, _transform: Matrix, _blend_mode: BlendMode) {}

    fn draw_tex(&mut self, _tex: &TextureReference, _color: Color, _h_flip: bool, _v_flip: bool, _transform: Matrix, _blend_mode: BlendMode) {}

    fn draw_slider(
        &mut self,
        _quad: [Vector2; 4],
        _transform: Matrix,

        _slider_data: SliderData,
        _slider_grids: Vec<GridCell>,
        _grid_cells: Vec<u32>,
        _line_segments: Vec<LineSegment>
    ) {}

    fn draw_flashlight(
        &mut self,
        _quad: [Vector2; 4],
        _transform: Matrix,
        _flashlight_data: FlashlightData
    ) {}

    fn add_emitter(&mut self, _emitter: EmitterReference) {}
    fn update_emitters(&mut self) {}
}