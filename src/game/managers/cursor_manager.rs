use crate::prelude::*;

pub struct CursorManager {
    /// position of the visible cursor
    pub pos: Vector2,

    /// should the cursor be visible?
    pub visible: bool,

    pub color: Color,

    /// should the mouse not follow the user's cursor.
    /// actually used inside game, not here
    pub replay_mode: bool,

    /// did the replay mode value change?
    /// needed so we know whether to show/hide the window cursor
    pub replay_mode_changed: bool,

    pub left_pressed: bool,
    pub right_pressed: bool,
}

impl CursorManager {
    pub fn new() -> Self {
        Self {
            pos: Vector2::zero(),
            visible: true,
            replay_mode: false,
            replay_mode_changed: false,
            color: Color::from_hex(&Settings::get_mut("CursorManager::new").cursor_color),

            left_pressed: false,
            right_pressed: false
        }
    }

    /// set replay mode.
    /// really just a helper
    #[allow(unused)]
    pub fn set_replay_mode(&mut self, val:bool) {
        if val != self.replay_mode {
            self.replay_mode = val;
            self.replay_mode_changed = true;
        }
    }

    pub fn set_cursor_pos(&mut self, pos:Vector2) {
        self.pos = pos;
    }

    pub fn draw(&mut self, list:&mut Vec<Box<dyn Renderable>>) {
        if !self.visible {return}

        let mut radius = 5.0;
        if self.left_pressed || self.right_pressed {
            radius *= 2.0;
        }

        let settings = Settings::get_mut("CursorManager::draw");

        let mut circle = Circle::new(
            self.color,
            -f64::MAX,
            self.pos,
            radius * settings.cursor_scale
        );
        if settings.cursor_border > 0.0 {
            let col = Color::from_hex(&settings.cursor_border_color);
            if col.a > 0.0 {
                circle.border = Some(Border::new(
                    col,
                    settings.cursor_border as f64
                ));
            }
        }
        
        list.push(Box::new(circle));
    }
}