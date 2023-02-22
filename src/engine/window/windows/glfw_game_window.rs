use crate::prelude::*;

use glfw::Context;
use piston::Event;
use piston::Input;
use piston::Window;
use piston::AdvancedWindow;


pub struct GlfwGameWindow {
    window: glfw_window::GlfwWindow,
    buffer_swappable: GlfwBufferable
}

impl GameWindowTrait for GlfwGameWindow {
    fn gl(&self) -> opengl_graphics::OpenGL { opengl_graphics::OpenGL::V4_5 }

    fn create(size: [u32; 2]) -> TatakuResult<Box<dyn GameWindowTrait>> where Self:Sized {
        let window:glfw_window::GlfwWindow = piston::WindowSettings::new("Tataku!", size)
            .graphics_api(opengl_graphics::OpenGL::V4_5)
            .build()
            .map_err(|e|TatakuError::String(e.to_string()))?;

        let ptr = window.window.window_ptr();
        #[cfg(target_os = "windows")] 
        unsafe {
            glfw::ffi::glfwSetWindowSizeCallback(ptr, Some(RESIZE_WINDOW));
            glfw::ffi::glfwSetWindowPosCallback(ptr, Some(REPOSITION_WINDOW));
        }
        
        Ok(Box::new(Self {
            buffer_swappable: GlfwBufferable(ptr),
            window,
        }))
    }

    fn set_size(&mut self, size: Vector2) {
        let size = [size.x as u32, size.y as u32];
        self.window.set_size(size);
    }
    fn get_size(&self) -> Vector2 {
        let size = self.window.size();
        Vector2::new(size.width as f64, size.height as f64)
    }
    fn get_draw_size(&self) -> Vector2 {
        let size = self.window.draw_size();
        Vector2::new(size.width as f64, size.height as f64)
    }

    fn set_icon(&mut self, image: image::RgbaImage) {
        self.window.window.set_icon(vec![image]);
    }

    fn set_vsync(&mut self, vsync: bool) {
        if vsync {
            self.window.glfw.set_swap_interval(glfw::SwapInterval::Sync(1))
        } else {
            self.window.glfw.set_swap_interval(glfw::SwapInterval::None)
        }
    }
    fn set_raw_mouse_input(&mut self, raw_mouse: bool) {
        self.window.window.set_raw_mouse_motion(raw_mouse)
    }

    fn request_attention(&mut self) {
        self.window.window.request_attention()
    }

    fn set_clipboard(&mut self, text: String) {
        self.window.window.set_clipboard_string(&text)
    }

    
    fn set_cursor_visible(&mut self, visible: bool) {
        if visible {
            self.window.window.set_cursor_mode(glfw::CursorMode::Normal)
        } else {
            self.window.window.set_cursor_mode(glfw::CursorMode::Hidden)
        }
    }

    fn apply_fullscreen(&mut self, monitor_num: usize) -> bool {
        self.window.glfw.with_connected_monitors(|_, monitors| {
            if let Some((_, monitor)) = monitors.iter().enumerate().find(|(n, _)|*n == monitor_num) {
                
                // if the monitor doesnt have a video mode (???) dont continue with fullscreen because we dont know what the resolution is
                let Some(mode) = monitor.get_video_mode() else { return false };
                let width = mode.width;
                let height = mode.height;

                self.window.window.set_monitor(glfw::WindowMode::FullScreen(monitor), 0, 0, width, height, None);

                true
            } else {
                false
            }
        })
    }

    fn apply_windowed(&mut self, [x, y]: [i32; 2]) {
        let size = self.window.size();
        let width  = size.width as u32;
        let height = size.height as u32;
        self.window.window.set_monitor(glfw::WindowMode::Windowed, x as i32, y as i32, width, height, None);
    }

    
    fn get_buffer_swappable(&mut self) -> &mut dyn BufferSwappable {
        &mut self.buffer_swappable
    }

    fn close(&mut self) {
        self.window.window.set_should_close(true);
    }

    fn get_monitors(&mut self) -> Vec<String> {
        self.window.glfw.with_connected_monitors(|_, monitors| {
            monitors.iter().filter_map(|m|m.get_name()).collect()
        })
    }

    fn poll_event(&mut self) -> Option<piston::Event> {
        self.window.poll_event()
    }

    
    fn check_controller_input(&mut self, event: &piston::Event) -> Option<GameEvent> {
        use piston::ControllerAxisEvent;
        use piston::ButtonEvent;

        if let Some(axis) = event.controller_axis_args() {
            let j_id = get_joystick_id(axis.id);
            let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());

            Some(GameEvent::ControllerEvent(event.clone(), name))
        } else if let Some(piston::input::Button::Controller(cb)) = event.button_args().map(|b|b.button) {
            // debug!("press: c: {}, b: {}", cb.id, cb.button);

            let j_id = get_joystick_id(cb.id);
            let name = self.window.glfw.get_joystick(j_id).get_name().unwrap_or("Unknown Name".to_owned());
            
            Some(GameEvent::ControllerEvent(event.clone(), name))
        } else {
            None
        }
    }
}

struct GlfwBufferable(*mut glfw::ffi::GLFWwindow);
impl BufferSwappable for GlfwBufferable {
    fn swap_buffers(&mut self) {
        unsafe {
            glfw::ffi::glfwSwapBuffers(self.0)
        }
    }
}


fn get_joystick_id(id: u32) -> glfw::JoystickId {
    use glfw::JoystickId::*;
    match id {
        0  => Joystick1,
        1  => Joystick2,
        2  => Joystick3,
        3  => Joystick4,
        4  => Joystick5,
        5  => Joystick6,
        6  => Joystick7,
        7  => Joystick8,
        8  => Joystick9,
        9  => Joystick10,
        10 => Joystick11,
        11 => Joystick12,
        12 => Joystick13,
        13 => Joystick14,
        14 => Joystick15,
        15 => Joystick16,
        _ => panic!("unknown joystick id: {}", id)
    }
}

// callbacks for windows because windows is bad

#[cfg(target_os = "windows")] 
pub static RESIZE_WINDOW:extern "C" fn(window: *mut glfw::ffi::GLFWwindow, i32, i32) = {
    extern "C" fn actual_callback(window: *mut glfw::ffi::GLFWwindow, w:i32, h:i32) {

        // generate a window event
        let draw_size = unsafe {
            let mut width = 0;
            let mut height = 0;
            glfw::ffi::glfwGetFramebufferSize(window, &mut width, &mut height);
            [width as u32, height as u32]
        };
        let window_size = [w as f64, h as f64];

        let _ = GAME_EVENT_SENDER.get().unwrap().try_send(
            GameEvent::WindowEvent(
                Event::Input(
                    Input::Resize(piston::ResizeArgs {
                        window_size,
                        draw_size,
                    }), 
                    None
                )
            )
        );

        let args = RenderArgs { 
            ext_dt: 0.0, 
            window_size,
            draw_size
        };

        let mut timer = Instant::now();

        // re-render
        let mut window = GlfwBufferable(window);
        render(&mut window, args, &mut timer);
    }
    actual_callback
};


#[cfg(target_os = "windows")] 
pub static REPOSITION_WINDOW:extern "C" fn(window: *mut glfw::ffi::GLFWwindow, i32, i32) = {
    extern "C" fn actual_callback(window: *mut glfw::ffi::GLFWwindow, x:i32, y:i32) {
        let draw_size = unsafe {
            let mut width = 0;
            let mut height = 0;
            glfw::ffi::glfwGetFramebufferSize(window, &mut width, &mut height);
            [width as u32, height as u32]
        };
        let window_size = unsafe {
            let mut width = 0;
            let mut height = 0;
            glfw::ffi::glfwGetWindowSize(window, &mut width, &mut height);
            [width as f64, height as f64]
        };

        {
            let mut settings = get_settings_mut!();
            settings.window_pos = [x, y];
            // println!("new pos: {:?}", settings.window_pos)
        }

        let args = RenderArgs { 
            ext_dt: 0.0, 
            window_size,
            draw_size
        };

        let mut timer = Instant::now();

        let mut window = GlfwBufferable(window);
        render(&mut window, args, &mut timer);
    }
    actual_callback
};

