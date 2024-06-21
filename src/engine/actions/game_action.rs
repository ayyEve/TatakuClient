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

    /// Add a notification
    AddNotification(Notification),

    /// Update the game's background
    UpdateBackground,
}

impl From<GameAction> for TatakuAction {
    fn from(value: GameAction) -> Self { Self::Game(value) }
}

impl core::fmt::Debug for GameAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quit => write!(f, "Quit"),
            Self::StartGame(_) => write!(f, "StartGame"),
            Self::ResumeMap(_) => write!(f, "ResumeMap"),
            Self::WatchReplay(_) => write!(f, "WatchReplay"),
            Self::SetValue(arg0, arg1) => f.debug_tuple("SetValue").field(arg0).field(arg1).finish(),
            Self::ViewScore(arg0) => write!(f, "ViewScore {}", arg0.hash()),
            Self::ViewScoreId(arg0) => f.debug_tuple("ViewScoreId").field(arg0).finish(),
            Self::HandleMessage(arg0) => f.debug_tuple("HandleMessage").field(arg0).finish(),
            Self::RefreshScores => write!(f, "RefreshScores"),
            Self::HandleEvent(arg0, arg1) => f.debug_tuple("HandleEvent").field(arg0).field(arg1).finish(),
            Self::AddNotification(arg0) => f.debug_tuple("AddNotification").field(arg0).finish(),
            Self::UpdateBackground => write!(f, "UpdateBackground"),
        }
    }
}