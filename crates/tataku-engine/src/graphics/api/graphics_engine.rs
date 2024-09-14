use crate::prelude::*;

pub type RenderTargetDraw = Box<dyn FnOnce(&mut dyn GraphicsEngine, Matrix)>;
pub type ScreenshotCallback = Box<dyn FnOnce((Vec<u8>, [u32; 2]))+Send+Sync>;

pub trait GraphicsEngine {
    fn is_dummy(&self) -> bool { false }

    /// resize the draw surface
    fn resize(&mut self, new_size: [u32; 2]);

    /// set the vsync mode
    fn set_vsync(&mut self, vsync: Vsync);


    fn create_render_target(
        &mut self, 
        size: [u32; 2], 
        clear_color: Color, 
        do_render: RenderTargetDraw,
    ) -> Option<RenderTarget>;
    fn update_render_target(
        &mut self, 
        target: RenderTarget, 
        do_render: RenderTargetDraw,
    );

    // texture things

    /// load a texture from bytes
    fn load_texture_bytes(&mut self, data: &[u8]) -> TatakuResult<TextureReference>;

    /// load a texture from RGBA bytes
    fn load_texture_rgba(&mut self, data: &[u8], size: [u32; 2]) -> TatakuResult<TextureReference>;

    /// free a texture
    fn free_tex(&mut self, tex: TextureReference);

    /// take a screenshot, returning the data via callback
    fn screenshot(&mut self, callback: ScreenshotCallback);



    // rendering

    /// start a render
    fn begin_render(&mut self);

    /// end the render
    fn end_render(&mut self);

    /// present the rendered surface
    fn present(&mut self) -> TatakuResult<()>;

    /// push a scissor to the scissor stack
    fn push_scissor(&mut self, scissor: [f32; 4]);

    /// pop a scissor from the scissor stack
    fn pop_scissor(&mut self);


    // drawing

    /// draw an arc with the center at 0,0
    fn draw_arc(&mut self, start: f32, end: f32, radius: f32, color: Color, resolution: u32, transform: Matrix, blend_mode: BlendMode);

    /// draw a circle with the center at 0,0
    fn draw_circle(&mut self, radius: f32, color: Color, border: Option<Border>, resolution: u32, transform: Matrix, blend_mode: BlendMode);

    /// draw a line from 0,0 to p
    fn draw_line(&mut self, p: Vector2, thickness: f32, color: Color, transform: Matrix, blend_mode: BlendMode);

    /// draw a rectangle
    fn draw_rect(&mut self, rect: [f32; 4], border: Option<Border>, shape: Shape, color: Color, transform: Matrix, blend_mode: BlendMode);

    /// draw a texture with top left at 0,0
    fn draw_tex(&mut self, tex: &TextureReference, color: Color, h_flip: bool, v_flip: bool, transform: Matrix, blend_mode: BlendMode);

    /// draw a slider
    fn draw_slider(
        &mut self,
        quad: [Vector2; 4],
        transform: Matrix,

        slider_data: SliderData,
        slider_grids: Vec<GridCell>,
        grid_cells: Vec<u32>,
        line_segments: Vec<LineSegment>
    );

    /// draw a flashlight
    fn draw_flashlight(
        &mut self,
        quad: [Vector2; 4],
        transform: Matrix,
        flashlight_data: FlashlightData
    );


    // particle engine stuff
    fn add_emitter(&mut self, emitter: Box<dyn EmitterReference>);
    fn update_emitters(&mut self);
}
