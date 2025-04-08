use crate::prelude::*;


#[derive(Clone, Debug)]
pub enum GameplayAction {
    /// Pause the game
    Pause,

    /// Resume the game
    Resume,

    /// Jump to a certain time
    JumpToTime {
        time: f32,
        skip_intro: bool,
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

    ApplyMods(ModManager),
    SetMode(GameplayMode),
    FitToArea(Bounds),

    /// The gameplay manager is requesting an update to the difficulty
    RequestDifficulty,
}

impl From<(GameplayId, GameplayAction)> for GameAction {
    fn from((id, action): (GameplayId, GameplayAction)) -> Self {
        Self::GameplayAction(id, action)
    }
}
