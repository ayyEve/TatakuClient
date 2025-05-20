use crate::prelude::*;

pub const CIRCLE_RADIUS_BASE:f32 = 64.0;
pub const OSU_NOTE_BORDER_SIZE:f32 = 2.0;

pub const FIELD_SIZE:Vector2 = Vector2::new(512.0, 384.0); // 4:3

#[derive(Copy, Clone)]
pub struct ScalingHelper {
    pub settings_offset: Vector2,

    /// scale setting in settings
    pub settings_scale: f32,

    /// window size to playfield size scale, scales by settings_scale
    pub scale: f32,

    /// window size from settings
    pub window_size: Vector2,

    /// cs size scaled
    pub cs: f32,

    /// border width
    pub border_width: f32,

    /// circle size
    pub circle_size: Vector2,

    /// should y coordinates be flipped over the playfield center?
    pub flip_vertical: bool,

    /// scaled playfield
    pub playfield: Bounds,

    /// playfield with note size padding
    pub playfield_with_padding: Bounds,
}
impl ScalingHelper {
    pub fn new_with_settings(settings: &OsuSettings, cs: f32, window_size: Vector2, flip_vertical: bool) -> Self {
        let (scale, offset) = settings.get_playfield();
        Self::new_offset_scale(cs, window_size, offset, scale, flip_vertical)
    }
    pub fn new_offset_scale(cs: f32, window_size: Vector2, settings_offset: Vector2, settings_scale: f32, flip_vertical: bool) -> Self {
        Self::new_offset_scale_custom_size(cs, window_size, settings_offset, settings_scale, flip_vertical, FIELD_SIZE)
    }

    pub fn new_offset_scale_custom_size(
        cs: f32,
        window_size: Vector2,
        settings_offset: Vector2,
        settings_scale: f32,
        flip_vertical: bool,
        playfield_size: Vector2,
    ) -> Self {
        let circle_size = CIRCLE_RADIUS_BASE;
        let border_size = OSU_NOTE_BORDER_SIZE;

        //
        let settings_offset = settings_offset + (playfield_size - FIELD_SIZE) / 2.0; // make sure the other thing is centered as well // what other thing ??

        let scale = (window_size / playfield_size).min_component() * settings_scale;

        // get where the playfield should be on the screen
        let pos = settings_offset + (window_size - playfield_size * scale) / 2.0;

        let cs_base = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * scale;
        let border_scaled = border_size * scale;
        let circle_size = Vector2::ONE * circle_size * scaled_cs;

        let playfield_with_padding = Bounds::new(
            pos - circle_size,
            playfield_size * scale + circle_size * 2.0
        );
        let playfield = Bounds::new(
            pos,
            playfield_size * scale,
        );

        Self {
            settings_offset,
            settings_scale,
            scale,
            window_size,
            cs: scaled_cs,
            border_width: border_scaled,
            circle_size,
            playfield_with_padding,
            flip_vertical,
            playfield,
        }
    }

    /// turn playfield (osu) coords into window coords
    pub fn scale_coords(&self, mut osu_coords: Vector2) -> Vector2 {
        if self.flip_vertical {
            osu_coords.y = FIELD_SIZE.y - osu_coords.y;
        }

        self.playfield.pos + osu_coords * self.scale
    }
    /// turn window coords into playfield coords
    pub fn descale_coords(&self, window_coords: Vector2) -> Vector2 {
        let mut v = (window_coords - self.playfield.pos) / self.scale;
        if self.flip_vertical { v.y = FIELD_SIZE.y - v.y }
        v
    }
}
