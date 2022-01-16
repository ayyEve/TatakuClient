use crate::prelude::*;

pub mod taiko;
pub mod mania;
pub mod catch;
pub mod standard;

pub const FIELD_SIZE:Vector2 = Vector2::new(512.0, 384.0); // 4:3

pub fn manager_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> Result<IngameManager, crate::errors::TaikoError> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let gamemode:Box<dyn GameMode> = match beatmap.playmode(playmode) {
        PlayMode::Standard => Box::new(standard::StandardGame::new(&beatmap)?),
        PlayMode::Taiko => Box::new(taiko::TaikoGame::new(&beatmap)?),
        PlayMode::Catch => Box::new(catch::CatchGame::new(&beatmap)?),
        PlayMode::Mania => Box::new(mania::ManiaGame::new(&beatmap)?),

        // TODO
        PlayMode::Adofai | //=> Box::new(taiko::TaikoGame::new(&beatmap)?),
        PlayMode::pTyping => return Err(TaikoError::GameMode(GameModeError::NotImplemented)),

        PlayMode::Unknown => return Err(TaikoError::GameMode(GameModeError::UnknownGameMode))
    };

    Ok(IngameManager::new(beatmap, gamemode))
}

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

    // /// scaled playfield
    // playfield_scaled: Rectangle,
    /// scaled playfield
    playfield_scaled_with_cs_border: Rectangle,
}
impl ScalingHelper {
    pub fn new(cs:f32, mode:PlayMode) -> Self {
        let window_size = Settings::window_size();

        let border_size;
        let circle_size;
        let settings_scale;
        let settings_offset;

        match mode {
            PlayMode::Standard => {
                let things = Settings::get_mut("ScalingHelper::new(PlayMode::Standard)").standard_settings.get_playfield();
                settings_scale = things.0;
                settings_offset = things.1;
                circle_size = standard::CIRCLE_RADIUS_BASE;

                border_size = standard::NOTE_BORDER_SIZE;
            },

            PlayMode::Catch => {
                let things = Settings::get_mut("ScalingHelper::new(PlayMode::Catch)").standard_settings.get_playfield();
                settings_scale = things.0;
                settings_offset = things.1;
                circle_size = standard::CIRCLE_RADIUS_BASE;

                border_size = standard::NOTE_BORDER_SIZE;
            },

            _ => {
                settings_scale = 0.0;
                settings_offset = Vector2::zero();
                circle_size = 0.0;
                border_size = 0.0;
            }
        };
            
        let scale = (window_size.y / FIELD_SIZE.y) * settings_scale;
        let scaled_pos_offset = (window_size - FIELD_SIZE * scale) / 2.0 + settings_offset;

        let cs_base = (1.0 - 0.7 * (cs as f64 - 5.0) / 5.0) / 2.0;
        let scaled_cs = cs_base * scale;

        let circle_size = Vector2::one() * circle_size * scaled_cs;

        let border_scaled = border_size * scale;

        // let playfield_scaled = Rectangle::new(
        //     [0.2, 0.2, 0.2, 0.5].into(),
        //     f64::MAX-4.0,
        //     scaled_pos_offset,
        //     FIELD_SIZE * scale,
        //     None
        // );

        let playfield_scaled_with_cs_border = Rectangle::new(
            [0.2, 0.2, 0.2, 0.5].into(),
            f64::MAX-4.0,
            scaled_pos_offset - circle_size,
            FIELD_SIZE * scale + circle_size,
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

            // playfield_scaled,
            playfield_scaled_with_cs_border
        }
    }

    /// turn playfield (osu) coords into window coords
    pub fn scale_coords(&self, osu_coords:Vector2) -> Vector2 {
        self.scaled_pos_offset + osu_coords * self.scale
    }
    /// turn window coords into playfield coords
    pub fn descale_coords(&self, window_coords: Vector2) -> Vector2 {
        (window_coords - self.scaled_pos_offset) / self.scale
    }

}