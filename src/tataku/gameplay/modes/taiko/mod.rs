mod taiko_game;
mod don_chan;
mod diff_calc;
mod taiko_info;
mod taiko_notes;
mod taiko_helpers;

pub use taiko_info::TaikoGameInfo as GameInfo;


pub(self) mod prelude {
    pub use super::taiko_game::*;
    pub use super::don_chan::*;
    pub use super::diff_calc::*;
    pub use super::taiko_info::*;
    pub use super::taiko_notes::*;
    pub use super::taiko_helpers::*;
}