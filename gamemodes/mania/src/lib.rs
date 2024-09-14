mod mania_game;
mod diff_calc;
mod info;
mod notes;
mod helpers;

pub use info::GAME_INFO;


mod prelude {
    pub use async_trait::async_trait;
    pub use tataku_engine::prelude::*;

    pub use super::mania_game::*;
    pub use super::diff_calc::*;
    pub use super::info::*;
    pub use super::notes::*;
    pub use super::helpers::*;
}