use crate::prelude::*;

#[derive(Clone)]
pub struct ManiaPlayfield {
    pub settings: ManiaPlayfieldSettings,
    pub bounds: Bounds,
    pub col_count: u8,
    pub total_width: f32,
}
impl ManiaPlayfield {
    pub fn new(mut settings: ManiaPlayfieldSettings, bounds: Bounds, col_count: u8) -> Self {
        let window_size = WindowSize::get().0;
        let total_width = col_count as f32 * settings.column_width;

        if bounds.size != window_size {
            // if we're not fullscreen, center the playfield
            settings.x_offset = bounds.pos.x + (total_width - bounds.size.x) / 2.0;
        }

        Self {
            settings, 
            bounds,
            col_count,
            total_width,
        }
    }

    /// y coordinate of the hit area
    pub fn hit_y(&self) -> f32 {
        self.bounds.pos.y + if self.upside_down {
            self.hit_pos
        } else {
            self.bounds.size.y - self.hit_pos
        }
    }

    /// leftmost x coordinate of the given column
    pub fn col_pos(&self, col: u8) -> f32 {
        let x_offset = self.x_offset + (self.bounds.size.x - self.total_width) / 2.0;

        x_offset + (self.column_width + self.column_spacing) * col as f32
    }
}


impl Deref for ManiaPlayfield {
    type Target = ManiaPlayfieldSettings;

    fn deref(&self) -> &Self::Target { 
        &self.settings 
    }
}
