use crate::prelude::*;

#[derive(Default)]
pub struct MouseInputHelper {
    mouse_pos: Vector2,
    window_focused: bool,
    raw_input: bool,
    system_cursor: bool,
    

    // // used for out of bounds checks
    // keep_in_bounds: bool,
}

impl MouseInputHelper {
    pub fn display_mouse_moved(&mut self, mouse_pos: Vector2) -> Option<Vector2> {
        // when the window isnt focused, we want to use the window mouse pos
        if (!self.window_focused || !self.raw_input) || self.system_cursor {
            self.mouse_pos = mouse_pos;
            return Some(mouse_pos)
        }
        
        None
    }

    pub fn device_mouse_moved(&mut self, delta: (f32, f32)) -> Option<Vector2> {
        // we only want to update the cursor if the window is focused and raw_input is enabled
        if self.window_focused && self.raw_input && !self.system_cursor {
            self.mouse_pos.x += delta.0;
            self.mouse_pos.y += delta.1;

            return Some(self.mouse_pos)
        }

        None
    }
    pub fn set_raw_input(&mut self, enabled: bool) {
        self.raw_input = enabled;
    }
    pub fn set_system_cursor(&mut self, enabled: bool) {
        self.system_cursor = enabled;
    }
    
    pub fn set_focus(&mut self, has_focus: bool) {
        self.window_focused = has_focus;
    }

    // pub fn check_bounds(&self, window: &winit::window::Window) -> Option<Vector2> {
    //     if self.keep_in_bounds { return None }

    //     let pos = window.outer_position().ok()?;
    //     let size = window.outer_size();
        
    //     let bounds = 

    // }

}
