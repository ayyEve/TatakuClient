mod mania_game;
mod diff_calc;
mod mania_info;
mod mania_notes;
mod mania_helpers;

pub use mania_info::GAME_INFO;


mod prelude {
    pub use async_trait::async_trait;
    pub use tataku_engine::prelude::*;

    pub use super::mania_game::*;
    pub use super::diff_calc::*;
    pub use super::mania_info::*;
    pub use super::mania_notes::*;
    pub use super::mania_helpers::*;

    pub use tracing::*;
}