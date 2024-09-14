mod utyping_game;
mod helpers;
mod diff_calc;
mod utyping_info;
mod utyping_notes;


pub use utyping_info::GAME_INFO;


mod prelude {
    pub use async_trait::async_trait;
    pub use tataku_engine::prelude::*;

    pub use super::helpers::*;
    pub use super::utyping_info::*;
    pub use super::utyping_notes::*;
    pub use super::utyping_game::UTypingGame;
    pub use super::diff_calc::UTypingDifficultyCalculator;
}