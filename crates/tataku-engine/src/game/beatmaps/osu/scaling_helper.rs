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

    pub fn new_transform(
        window_size: Vector2,
        settings_offset: Vector2,
        settings_scale: f32,
        flip_vertical: bool,
        playfield_size: Option<Vector2>,
    ) -> Transform {
        let playfield_size = playfield_size.unwrap_or(FIELD_SIZE);

        //
        let settings_offset = settings_offset + (playfield_size - FIELD_SIZE) / 2.0; // make sure the other thing is centered as well // what other thing ??

        let scale = (window_size / playfield_size).min_component() * settings_scale;

        // get where the playfield should be on the screen
        let pos = settings_offset + (window_size - playfield_size * scale) / 2.0;

        Transform::new(
            pos + if flip_vertical { Vector2::new(0.0, window_size.y) } else { Vector2::ZERO },
            Vector2::new(scale, scale * if flip_vertical { -1.0 } else { 1.0 }), // FIXME:
            0.0,
            Vector2::ZERO
        )
    }

    pub fn transform_padded(
        mut transform: Transform,
        cs: f32,
        playfield_size: Option<Vector2>,
    ) -> Transform {
        let playfield_size = playfield_size.unwrap_or(FIELD_SIZE) * transform.scale.x;

        let cs_base = (1.0 - 0.7 * (cs - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * transform.scale.x;
        let circle_size = Vector2::ONE * CIRCLE_RADIUS_BASE * scaled_cs;

        transform.pos -= circle_size;
        transform.scale = (playfield_size + circle_size * 2.0) / playfield_size;

        transform
    }


    pub fn new_with_settings(settings: &OsuSettings, cs: f32, window_size: Vector2, flip_vertical: bool) -> Self {
        let (scale, offset) = settings.get_playfield();
        Self::new_offset_scale(cs, window_size, offset, scale, flip_vertical)
    }
    pub fn new_with_settings_custom_size(settings: &OsuSettings, cs: f32, window_size: Vector2, flip_vertical: bool, size: Vector2) -> Self {
        let (scale, offset) = settings.get_playfield();
        Self::new_offset_scale_custom_size(cs, window_size, offset, scale, flip_vertical, size)
    }
    pub fn new_offset_scale(cs: f32, window_size: Vector2, settings_offset: Vector2, settings_scale: f32, flip_vertical: bool) -> Self {
        Self::new_offset_scale_custom_size(cs, window_size, settings_offset, settings_scale, flip_vertical, FIELD_SIZE)
    }

    /// makes a lot of assumptions about things
    pub fn fit_to_playfield(
        playfield: PlayfieldNonsense,
        flip_vertical: bool,
    ) -> Self {
        let playfield_with_padding = Bounds::new(
            playfield.bounds.pos - playfield.circle_size,
            playfield.bounds.size + playfield.circle_size * 2.0
        );

        Self {
            settings_offset: Vector2::ZERO,
            settings_scale: 1.0,
            scale: playfield.scale,
            window_size: playfield.bounds.size,
            cs: 0.0,
            border_width: 0.0,
            circle_size: playfield.circle_size,
            playfield_with_padding,
            flip_vertical,
            playfield: playfield.bounds,
        }
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
            osu_coords.y = FIELD_SIZE.y - osu_coords.y
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

#[derive(Copy, Clone, Debug, Default)]
pub struct PlayfieldNonsense {
    pub bounds: Bounds,
    pub scale: f32,
    pub circle_size: Vector2,
    pub flip_vertical: bool
}
impl PlayfieldNonsense {
    pub fn new(
        bounds: Bounds, 
        scale: f32, 
        circle_size: Vector2,
        flip_vertical: bool,
    ) -> Self {
        Self {
            bounds,
            scale,
            circle_size,
            flip_vertical,
        }
    }

    pub fn new_simple(bounds: Bounds) -> Self {
        Self {
            bounds,
            scale: 1.0,
            ..Default::default()
        }
    }
}