use image::RgbaImage;
use crate::prelude::*;


pub trait GameWindowTrait {
    fn gl(&self) -> opengl_graphics::OpenGL;

    fn create(size: [u32; 2]) -> TatakuResult<Box<dyn GameWindowTrait>> where Self:Sized;
    fn set_icon(&mut self, image: RgbaImage);

    fn set_size(&mut self, size: Vector2);
    fn get_size(&self) -> Vector2;
    fn get_draw_size(&self) -> Vector2;


    fn set_vsync(&mut self, vsync: bool);
    fn set_raw_mouse_input(&mut self, raw_mouse: bool);

    fn set_cursor_visible(&mut self, visible: bool);

    fn set_clipboard(&mut self, text: String);


    fn get_monitors(&mut self) -> Vec<String>;
    /// set fullscreen, return false if failed
    fn apply_fullscreen(&mut self, monitor: usize) -> bool;
    fn apply_windowed(&mut self);

    fn request_attention(&mut self);

    fn get_buffer_swappable(&mut self) -> &mut dyn BufferSwappable;

    fn close(&mut self);

    fn poll_event(&mut self) -> Option<piston::Event>; 


    fn check_controller_input(&mut self, event: &piston::Event) -> Option<GameEvent>;
}

pub trait BufferSwappable {
    fn swap_buffers(&mut self);
}
