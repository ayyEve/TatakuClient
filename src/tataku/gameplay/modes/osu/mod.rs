mod osu_math;
mod osu_game;
mod osu_info;
mod osu_notes;
mod diff_calc;
mod osu_helpers;

pub use osu_info::OsuGameInfo as GameInfo;

/// import helper
pub(self) mod prelude {
    pub use super::osu_math::*;
    pub use super::osu_game::*;
    pub use super::osu_info::*;
    pub use super::osu_notes::*;
    pub use super::osu_helpers::*;
    pub use super::diff_calc::OsuDifficultyCalculator;
}