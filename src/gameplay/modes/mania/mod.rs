mod mania;
mod diff_calc;
mod mania_notes;
mod mania_hit_judgments; 

pub use mania::calc_acc;
pub use mania::ManiaGame as Game;
pub(super) use mania_hit_judgments::*;
pub use diff_calc::ManiaDifficultyCalculator as DiffCalc;