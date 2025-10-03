use crate::*;

pub type RenderTargetDraw = Arc<dyn Fn(&mut dyn DrawEngine, Matrix) + Send + Sync>;
pub type ScreenshotCallback = Box<dyn FnOnce((Vec<u8>, [u32; 2])) + Send + Sync>;

pub trait RenderingEngine {
    fn is_dummy(&self) -> bool { false }

    fn dump_atlas(&self, _path: &str) {}

    /// set if blur should be enabled or not
    fn set_blur(&mut self, enabled: bool);

    /// resize the draw surface
    fn resize(&mut self, new_size: [u32; 2]);

    /// set the vsync mode
    fn set_vsync(&mut self, vsync: Vsync);

    /// Get the list of avaiable vsync modes
    fn vsync_modes(&self) -> Vec<Vsync>;


    // texture things

    /// load a texture from file bytes (ie .png file)
    fn load_texture_bytes(&mut self, data: &[u8]) -> tataku::Result<TextureReference>;

    /// load a texture from RGBA
    fn load_texture_rgba(&mut self, data: &[u8], size: [u32; 2]) -> tataku::Result<TextureReference>;

    /// free a texture
    fn free_tex(&mut self, tex: TextureReference, defer_until_next_draw: bool);

    /// take a screenshot, returning the data via callback
    fn screenshot(&mut self, callback: ScreenshotCallback);

    // rendering

    /// start a render
    fn begin_render(&mut self);

    /// end the render
    fn end_render(&mut self);

    /// present the rendered surface
    fn present(&mut self) -> tataku::Result<()>;

    // particle engine stuff
    fn add_emitter(&mut self, emitter: EmitterReference);
    fn update_emitters(&mut self);

    fn with_renderer(&mut self, draw: &dyn Fn(&mut dyn DrawEngine));
}

pub trait DrawEngine {
    /// push a scissor to the scissor stack
    fn push_scissor(&mut self, scissor: [f32; 4]);

    /// pop a scissor from the scissor stack
    fn pop_scissor(&mut self);

    // drawing

    /// draw an arc with the center at 0,0
    #[allow(clippy::too_many_arguments)]
    fn draw_arc(
        &mut self,
        start: f32,
        end: f32,
        radius: f32,
        color: Color,
        border: Option<Border>,
        resolution: u32,
        transform: Matrix,
        blend_mode: BlendMode,
    );

    /// draw a circle with the center at 0,0
    fn draw_circle(
        &mut self,
        radius: f32,
        color: Color,
        border: Option<Border>,
        resolution: u32,
        transform: Matrix,
        blend_mode: BlendMode,
    );

    /// draw a line from 0,0 to p
    fn draw_line(
        &mut self,
        p: Vector2,
        thickness: f32,
        color: Color,
        transform: Matrix,
        blend_mode: BlendMode,
    );

    /// draw a rectangle
    fn draw_rect(
        &mut self,
        rect: [f32; 4],
        border: Option<Border>,
        shape: Shape,
        color: Color,
        transform: Matrix,
        blend_mode: BlendMode,
    );

    /// draw a texture with top left at 0,0
    fn draw_tex(
        &mut self,
        tex: TextureDraw<'_>,
        transform: Matrix,
        blend_mode: BlendMode,
    );

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

    fn draw_gaussian_blur(
        &mut self,
        bounds: Bounds,
        sigma: f32,
        rounds: u32,
    );

    fn draw_box_blur(
        &mut self,
        bounds: Bounds,
        size: u32,
    );

    fn draw_text(
        &mut self,
        transform: Matrix,
        blend_mode: BlendMode,
        layout: &parley::Layout<tataku_engine_common::prelude::Color>,
    );

    // render targets
    fn create_render_target(
        &mut self,
        data: &mut RenderTargetData,
        do_render: RenderTargetDraw,
    );
    fn update_render_target(
        &mut self,
        data: &RenderTargetData,
        do_render: RenderTargetDraw,
    );
}


#[derive(Copy, Clone, Debug)]
pub struct TextureDraw<'a> {
    pub tex: &'a TextureReference,
    pub color: Color,
    pub flip: ImageFlip,
}
impl<'a> TextureDraw<'a> {
    pub fn new(
        tex: &'a TextureReference,
        color: Color,
    ) -> Self {
        Self {
            tex,
            color,
            flip: ImageFlip::None
        }
    }

    pub fn with_flip(mut self, flip: ImageFlip) -> Self {
        self.flip ^= flip;
        self
    }

    pub fn with_hflip(mut self, hflip: bool) -> Self {
        self.flip = ImageFlip::new(
            hflip,
            self.flip.flip_v(),
        );
        self
    }
    pub fn with_vflip(mut self, vflip: bool) -> Self {
        self.flip = ImageFlip::new(
            self.flip.flip_h(),
            vflip,
        );
        self
    }
}
