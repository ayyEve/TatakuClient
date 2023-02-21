use ayyeve_piston_ui::prelude::piston::Window;
use crate::prelude::*;


pub struct GlutinGameWindow {
    window: glutin_window::GlutinWindow
}

impl GameWindowTrait for GlutinGameWindow {
    fn gl(&self) -> opengl_graphics::OpenGL { opengl_graphics::OpenGL::V4_5 }

    fn create(size: [u32; 2]) -> TatakuResult<Box<dyn GameWindowTrait>> where Self:Sized {
        let window:glutin_window::GlutinWindow = piston::WindowSettings::new("Tataku!", size)
            .graphics_api(opengl_graphics::OpenGL::V4_5)
            .build()
            .map_err(|e|TatakuError::String(e.to_string()))?;

        Ok(Box::new(Self {
            window
        }))
    }

    fn set_icon(&mut self, image: image::RgbaImage) {
        let width = image.width();
        let height = image.height();
        
        match winit::window::Icon::from_rgba(image.into_vec(), width, height) {
            Ok(icon) => {
                self.window.ctx.window().set_window_icon(Some(icon.clone()));
                
                #[cfg(target_os="windows")] {
                    use winit::platform::windows::WindowExtWindows;
                    self.window.ctx.window().set_taskbar_icon(Some(icon));
                }
            },
            Err(e) => warn!("error setting window icon: {}", e)
        }
    }


    fn set_size(&mut self, size: Vector2) {
        let size = [size.x as u32, size.y as u32];
        self.window.ctx.resize(size.into());
    }
    fn get_size(&self) -> Vector2 {
        let size = self.window.size();
        Vector2::new(size.width as f64, size.height as f64)
    }
    fn get_draw_size(&self) -> Vector2 {
        let size = self.window.draw_size();
        Vector2::new(size.width as f64, size.height as f64)
    }

    
    fn set_vsync(&mut self, _vsync: bool) {}
    fn set_raw_mouse_input(&mut self, _raw_mouse: bool) {}

    fn set_cursor_visible(&mut self, visible: bool) {
        self.window.ctx.window().set_cursor_visible(visible)
    }

    fn set_clipboard(&mut self, text: String) {
        use clipboard::{ClipboardProvider, ClipboardContext};

        let ctx:Result<ClipboardContext, Box<dyn std::error::Error>> = ClipboardProvider::new();
        match ctx {
            Ok(mut ctx) => if let Err(e) = ctx.set_contents(text) {
                error!("[Clipboard] Error: {:?}", e);
            }
            Err(e) => error!("[Clipboard] Error: {:?}", e),
        }
    }


    fn request_attention(&mut self) {
        self.window.ctx.window().request_user_attention(Some(winit::window::UserAttentionType::Informational))
    }

    fn apply_fullscreen(&mut self, monitor_num: usize) -> bool {
        if let Some((_, monitor)) = self.window.ctx.window().available_monitors().enumerate().find(|(n, _)|*n == monitor_num) {
            self.window.ctx.window().set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor))));
            true
        } else {
            false
        }
    }

    fn apply_windowed(&mut self, [x, y]: [i32; 2]) {
        self.window.ctx.window().set_fullscreen(None);
        let pos = winit::dpi::PhysicalPosition::new(x, y);
        self.window.ctx.window().set_outer_position(pos)
    }

    fn get_buffer_swappable(&mut self) -> &mut dyn BufferSwappable {
        self
    }

    fn close(&mut self) {
        self.window.set_should_close(true);
    }

    fn get_monitors(&mut self) -> Vec<String> {
        self.window.ctx.window().available_monitors().filter_map(|m|m.name()).collect()
    }


    fn poll_event(&mut self) -> Option<piston::Event> {
        self.window.poll_event()
    }

    
    fn check_controller_input(&mut self, _event: &piston::Event) -> Option<GameEvent> {
        None
    }
}

impl BufferSwappable for GlutinGameWindow {
    fn swap_buffers(&mut self) {
        self.window.swap_buffers();
    }
}

