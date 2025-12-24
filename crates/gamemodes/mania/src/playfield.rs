use crate::prelude::*;

#[derive(Clone, Default)]
pub struct ManiaPlayfield {
    pub settings: ManiaPlayfieldSettings,
    pub bounds: tataku::Bounds,
    // pub col_count: u8,
    pub total_width: f32,

    /// bullshit peppy fuck
    pub skin_hit_pos: f32,

    // full_window: bool,
}
impl ManiaPlayfield {
    pub fn new(
        mut settings: ManiaPlayfieldSettings,
        bounds: tataku::Bounds,
        col_count: u8,
        skin_hit_pos: f32,
        full_window: bool,
    ) -> Self {
        let total_width = col_count as f32 * settings.column_width;

        if !full_window {
            // if we're not fullscreen, center the playfield
            settings.x_offset = bounds.pos.x + (total_width - bounds.size.x) / 2.0;
        }

        Self {
            settings,
            bounds,
            // col_count,
            total_width,

            skin_hit_pos,
            // full_window
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
