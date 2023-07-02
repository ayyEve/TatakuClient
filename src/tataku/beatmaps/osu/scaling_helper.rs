use super::super::prelude::*;


pub const CIRCLE_RADIUS_BASE:f32 = 64.0;
pub const OSU_NOTE_BORDER_SIZE:f32 = 2.0;

pub const FIELD_SIZE:Vector2 = Vector2::new(512.0, 384.0); // 4:3

#[derive(Copy, Clone)]
pub struct ScalingHelper {
    /// scale setting in settings
    pub settings_scale: f32,
    /// playfield offset in settings
    pub settings_offset: Vector2,

    /// window size to playfield size scale, scales by settings_scale
    pub scale: f32,

    /// window size from settings
    pub window_size: Vector2,

    /// scaled pos offset for the playfield
    pub scaled_pos_offset: Vector2,

    /// cs size scaled
    pub scaled_cs: f32,

    /// border size scaled
    pub border_scaled: f32,

    pub scaled_circle_size: Vector2,

    pub flip_vertical: bool,

    // /// scaled playfield
    // playfield_scaled: Rectangle,
    /// scaled playfield
    pub playfield_scaled_with_cs_border: Rectangle,
}
impl ScalingHelper {
    pub async fn new(cs:f32, window_size: Vector2, flip_vertical: bool) -> Self {
        let settings = Settings::get().osu_settings.clone();
        Self::new_with_settings(&settings, cs, window_size, flip_vertical)
    }
    pub fn new_with_settings(settings: &OsuSettings, cs:f32, window_size: Vector2, flip_vertical: bool) -> Self {
        let (scale, offset) = settings.get_playfield();
        Self::new_offset_scale(cs, window_size, offset, scale, flip_vertical)
    }
    pub fn new_with_settings_custom_size(settings: &OsuSettings, cs:f32, window_size: Vector2, flip_vertical: bool, size: Vector2) -> Self {
        let (scale, offset) = settings.get_playfield();
        Self::new_offset_scale_custom_size(cs, window_size, offset, scale, flip_vertical, size)
    }
    pub fn new_offset_scale(cs:f32, window_size: Vector2, settings_offset: Vector2, settings_scale: f32, flip_vertical: bool) -> Self {
        Self::new_offset_scale_custom_size(cs, window_size, settings_offset, settings_scale, flip_vertical, FIELD_SIZE)
    }

    pub fn new_offset_scale_custom_size(cs:f32, window_size: Vector2, settings_offset: Vector2, settings_scale: f32, flip_vertical: bool, playfield_size: Vector2) -> Self {
        let circle_size = CIRCLE_RADIUS_BASE;
        let border_size = OSU_NOTE_BORDER_SIZE;

        let settings_offset = settings_offset + (playfield_size-FIELD_SIZE) / 2.0; // make sure the other thing is centered as well
        
        let scale = (window_size.y / playfield_size.y) * settings_scale;
        let scaled_pos_offset = (window_size - playfield_size * scale) / 2.0 + settings_offset;

        let cs_base = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * scale;
        let border_scaled = border_size * scale;
        let circle_size = Vector2::ONE * circle_size * scaled_cs;

        let playfield_scaled_with_cs_border = Rectangle::new(
            scaled_pos_offset - circle_size,
            playfield_size * scale + circle_size * 2.0,
            Color::new(0.2, 0.2, 0.2, 0.5),
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
