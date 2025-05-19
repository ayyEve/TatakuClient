mod osu_math;
mod osu_game;
mod osu_info;
mod osu_notes;
mod diff_calc;
mod osu_cursor;
mod osu_helpers;
mod osu_settings;

pub use osu_info::GAME_INFO;

/// import helper
mod prelude {
    pub use tataku_engine::prelude::*;

    pub use super::osu_math::*;
    pub use super::osu_game::*;
    pub use super::osu_info::*;
    pub use super::osu_notes::*;
    pub use super::osu_cursor::*;
    pub use super::osu_helpers::*;
    pub use super::osu_settings::*;
    pub use super::diff_calc::OsuDifficultyCalculator;
}