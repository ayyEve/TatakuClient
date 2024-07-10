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

    SetHitsoundsEnabled(bool),
}
