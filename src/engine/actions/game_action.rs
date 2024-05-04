use crate::prelude::*;

pub enum GameAction {
    /// Fully quit the game
    Quit,

    /// Start a game with the provided ingame manager
    StartGame(Box<IngameManager>),

    /// Resume a map
    ResumeMap(Box<IngameManager>),

    /// Watch a replay
    WatchReplay(Box<Replay>),

    /// Update a value 
    SetValue(String, TatakuValue),

    /// Open a score in the score menu
    ViewScore(IngameScore),

    /// Open a score in the score menu
    ViewScoreId(usize),

    /// Handle a message
    HandleMessage(Message),

    /// Refresh the scores list
    RefreshScores,

    /// Handle an event
    HandleEvent(TatakuEventType, Option<TatakuValue>),
}

impl From<GameAction> for TatakuAction {
    fn from(value: GameAction) -> Self { Self::Game(value) }
}
