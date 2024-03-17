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
    SetValue(String, CustomElementValue),

    /// Open a score in the score menu
    ViewScore(IngameScore),

    /// Handle a message
    HandleMessage(Message),
}

impl From<GameAction> for TatakuAction {
    fn from(value: GameAction) -> Self { Self::Game(value) }
}

