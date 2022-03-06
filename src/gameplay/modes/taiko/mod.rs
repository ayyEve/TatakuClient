mod taiko;
mod taiko_notes;
mod diff_calc;

use taiko::*;
use taiko_notes::*;

pub use taiko::calc_acc;
pub use taiko::TaikoGame as Game;
pub use diff_calc::TaikoDifficultyCalculator as DiffCalc;