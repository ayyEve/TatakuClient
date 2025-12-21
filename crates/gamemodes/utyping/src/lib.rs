mod game;
mod info;
mod notes;
mod helpers;
mod settings;
mod diff_calc;

pub use info::GAME_INFO;


mod prelude {
    pub use tataku_engine as engine;
    pub use tataku_engine_common::common::*;

    pub use super::helpers::*;
    pub use super::info::*;
    pub use super::notes::*;
    pub use super::settings::*;
    pub use super::game::UTypingGame;
    pub use super::diff_calc::UTypingDifficultyCalculator;
}