mod taiko;
mod don_chan;
mod diff_calc;
mod taiko_notes;

use taiko::*;
use taiko_notes::*;

pub use don_chan::*;
pub use taiko::calc_acc;
pub use taiko::TaikoGame as Game;
pub use diff_calc::TaikoDifficultyCalculator as DiffCalc;