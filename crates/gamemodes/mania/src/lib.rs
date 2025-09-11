mod mania_game;
mod diff_calc;
mod info;
mod notes;
mod helpers;
mod mania_settings;

pub use info::GAME_INFO;

mod prelude {
    pub use engine::input;
    pub use tataku_engine as engine;
    pub use tataku_engine::graphics;
    pub use tataku_client_common::common::*;

    pub use super::mania_game::*;
    pub use super::mania_settings::*;
    pub use super::diff_calc::*;
    pub use super::info::*;
    pub use super::notes::*;
    pub use super::helpers::*;
}