use crate::prelude::*;
use super::super::prelude::*;

#[derive(Copy, Clone)]
pub struct ScalingHelper {
    /// scale setting in settings
    pub settings_scale: f64,
    /// playfield offset in settings
    pub settings_offset: Vector2,

    /// window size to playfield size scale, scales by settings_scale
    pub scale: f64,

    /// window size from settings
    pub window_size: Vector2,

    /// scaled pos offset for the playfield
    pub scaled_pos_offset: Vector2,

    /// cs size scaled
    pub scaled_cs: f64,

    /// border size scaled
    pub border_scaled: f64,

    pub scaled_circle_size: Vector2,

    pub flip_vertical: bool,

    // /// scaled playfield
    // playfield_scaled: Rectangle,
    /// scaled playfield
    pub playfield_scaled_with_cs_border: Rectangle,
}
impl ScalingHelper {
    pub async fn new(cs:f32, window_size: Vector2, flip_vertical: bool) -> Self {
        let things = get_settings!().standard_settings.get_playfield();
        let settings_scale = things.0;
        let settings_offset = things.1;
        Self::new_offset_scale(cs, window_size, settings_offset, settings_scale, flip_vertical)
    }

    pub fn new_offset_scale(cs:f32, window_size: Vector2, settings_offset: Vector2, settings_scale: f64, flip_vertical: bool) -> Self {
        let circle_size = CIRCLE_RADIUS_BASE;
        let border_size = NOTE_BORDER_SIZE;
        
        let scale = (window_size.y / FIELD_SIZE.y) * settings_scale;
        let scaled_pos_offset = (window_size - FIELD_SIZE * scale) / 2.0 + settings_offset;

        let cs_base = (1.0 - 0.7 * (cs as f64 - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * scale;

        let circle_size = Vector2::ONE * circle_size * scaled_cs;

        let border_scaled = border_size * scale;

        let playfield_scaled_with_cs_border = Rectangle::new(
            [0.2, 0.2, 0.2, 0.5].into(),
            f64::MAX-4.0,
            scaled_pos_offset - circle_size,
            FIELD_SIZE * scale + circle_size * 2.0,
            None
        );

        Self {
            settings_scale,
            settings_offset,
            scale,
            window_size,
            scaled_pos_offset,
            scaled_cs,
            border_scaled,
            scaled_circle_size: circle_size,
            playfield_scaled_with_cs_border,
            flip_vertical
        }
    }

    /// turn playfield (osu) coords into window coords
    pub fn scale_coords(&self, mut osu_coords:Vector2) -> Vector2 {
        if self.flip_vertical {
            osu_coords.y = FIELD_SIZE.y - osu_coords.y
        }

        self.scaled_pos_offset + osu_coords * self.scale
    }
    /// turn window coords into playfield coords
    pub fn descale_coords(&self, window_coords: Vector2) -> Vector2 {
        let mut v = (window_coords - self.scaled_pos_offset) / self.scale;
        if self.flip_vertical { v.y = FIELD_SIZE.y - v.y }
        v
    }
}
