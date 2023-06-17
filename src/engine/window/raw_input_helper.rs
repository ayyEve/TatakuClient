use crate::prelude::*;

#[derive(Default)]
pub struct MouseInputHelper {
    mouse_pos: Vector2,
    window_focused: bool,
    raw_input: bool,
    system_cursor: bool,


    // in_bounds: bool,
    
    // // used for out of bounds checks
    // keep_in_bounds: bool,
}

impl MouseInputHelper {
    pub fn set_raw_input(&mut self, enabled: bool) {
        self.raw_input = enabled;
    }
    pub fn set_system_cursor(&mut self, enabled: bool) {
        self.system_cursor = enabled;
    }
    pub fn set_focus(&mut self, has_focus: bool, window: &winit::window::Window) {
        self.window_focused = has_focus;

        if has_focus {
            let _ = window.set_cursor_position(winit::dpi::LogicalPosition::new(self.mouse_pos.x as f64, self.mouse_pos.y as f64));
        }
    }

    pub fn display_mouse_moved(&mut self, mouse_pos: Vector2) -> Option<Vector2> {
        // when the window isnt focused, we want to use the window mouse pos
        if (!self.window_focused || !self.raw_input) || self.system_cursor {
            self.mouse_pos = mouse_pos;
            return Some(mouse_pos)
        }
        
        None
    }

    pub fn device_mouse_moved(&mut self, delta: (f32, f32), window: &winit::window::Window) -> Option<Vector2> {
        // we only want to update the cursor if the window is focused and raw_input is enabled
        if self.window_focused && self.raw_input && !self.system_cursor {
            // self.mouse_pos.x += delta.0;
            // self.mouse_pos.y += delta.1;

            let size = window.inner_size();
            self.mouse_pos.x = (self.mouse_pos.x + delta.0).clamp(0.0, size.width as f32);
            self.mouse_pos.y = (self.mouse_pos.y + delta.1).clamp(0.0, size.height as f32);

            return Some(self.mouse_pos)
        }

        None
    }

    pub fn reset_cursor_pos(&mut self, window: &winit::window::Window) {
        let size = window.inner_size();
        let pos = Vector2::new(size.width as f32, size.height as f32) / 2.0;
        
        if let Ok(_) = window.set_cursor_position(winit::dpi::LogicalPosition::new(pos.x as f64, pos.y as f64)) {
            self.mouse_pos = pos;
        }
    }


    // /// returns whether we should reset the mouse cursor
    // pub fn check_bounds(&mut self, window: &winit::window::Window) -> bool {
    //     // never touch the mouse pos if we arent doing raw input
    //     if !self.raw_input { return false }

    //     // if we always want the cursor to be within the window
    //     if self.keep_in_bounds { return true }

    //     let Ok(pos) = window.outer_position() else { return false };
    //     let size = window.outer_size();
        
    //     let bounds = Rectangle::bounds_only(
    //         Vector2::new(pos.x as f32, pos.y as f32),
    //         Vector2::new(size.width as f32, size.height as f32)
    //     );

    //     let contains = bounds.contains(self.mouse_pos);

    //     if contains && !self.in_bounds {
    //         println!("cursor back in bounds");
    //         self.in_bounds = true;
    //     }

    //     if !contains && self.in_bounds {
    //         println!("cursor left bounds");
    //         self.in_bounds = false;
    //         let _ = window.set_cursor_position(winit::dpi::LogicalPosition::new(self.mouse_pos.x as f64, self.mouse_pos.y as f64));
    //     }

    //     self.in_bounds
    // }

}
