use crate::*;
use common::types::replays::ReplayAction;


#[derive(Clone, Debug)]
pub enum GameplayAction {
    /// Pause the game
    Pause,

    /// Resume the game
    Resume,

    /// Jump to a certain time
    JumpToTime {
        time: f32,
    },

    /// Add a replay action
    AddReplayAction {
        /// Action to add
        action: ReplayAction,

        /// Should this action be saved to the replay?
        ///
        /// Helpful for spammy actions to keep filesize low (ie cursor position)
        should_save: bool,
    },

    ApplyMods(gameplay::mods::Mods),
    FitToArea(tataku::Bounds),

    /// The gameplay manager is requesting an update to the difficulty
    RequestDifficulty,
}

impl From<(actions::game::GameplayId, GameplayAction)> for actions::game::GameAction {
    fn from((id, action): (actions::game::GameplayId, GameplayAction)) -> Self {
        Self::GameplayAction(id, action)
    }
}
