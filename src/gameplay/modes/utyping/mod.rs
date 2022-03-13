mod u_typing;
mod u_typing_notes;
mod diff_calc;

use u_typing::*;
use u_typing_notes::*;

pub use u_typing::calc_acc;
pub use u_typing::UTypingGame as Game;
pub use diff_calc::UTypingDifficultyCalculator as DiffCalc;