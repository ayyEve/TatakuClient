mod taiko_game;
mod don_chan;
mod diff_calc;
mod taiko_info;
mod taiko_notes;
mod taiko_helpers;
mod taiko_settings;

pub use taiko_info::GAME_INFO;

mod prelude {
    pub use tataku_engine as engine;
    pub use tataku_engine_common::common::*;

    pub use super::don_chan::*;
    pub use super::diff_calc::*;
    pub use super::taiko_info::*;
    pub use super::taiko_game::*;
    pub use super::taiko_notes::*;
    pub use super::taiko_helpers::*;
    pub use super::taiko_settings::*;
}
