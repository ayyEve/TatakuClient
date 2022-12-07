mod mania_game;
mod diff_calc;
mod mania_info;
mod mania_notes;

pub use mania_info::ManiaGameInfo as GameInfo;


pub(self) mod prelude {
    pub use super::mania_game::*;
    pub use super::diff_calc::*;
    pub use super::mania_info::*;
    pub use super::mania_notes::*;
}