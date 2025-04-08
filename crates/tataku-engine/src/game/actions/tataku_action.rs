use crate::prelude::*;

/// An action from a menu (or dialog) to tell the game to do something
#[derive(Default, Debug2)]
pub enum TatakuAction {
    /// Don't do anything (this is a helper)
    #[default] None,

    /// Perform an audio action
    Audio(AudioAction),

    /// Perform a menu operation
    Menu(MenuAction),

    /// Perform a game operation
    Game(Box<GameAction>),

    /// Perform a game operation
    Online(OnlineAction),

    /// Perform a beatmap operation
    Beatmap(BeatmapAction),

    /// Perform an operation on the current song
    Song(SongAction),

    /// Perform a mods action
    Mods(ModAction),

    /// Perform an action on the Ui
    Ui(UiAction),

    /// Perform a multiplayer action
    Multiplayer(MultiplayerAction),

    /// Perform a task action
    Task(TaskAction),

    /// Perform a cursor action
    CursorAction(CursorAction),

    /// Perform a window action
    WindowAction(WindowAction),

    /// Download a file
    Download(#[debug(skip)] Box<Downloadable>),

    /// Handle an event
    Event(TatakuIntegrationEvent),

    /// Handle multiple actions
    Multiple(Vec<Self>)
}

impl From<Notification> for TatakuAction {
    fn from(value: Notification) -> Self {
        Self::Game(Box::new(GameAction::AddNotification(value)))
    }
}

impl From<TatakuIntegrationEvent> for TatakuAction {
    fn from(value: TatakuIntegrationEvent) -> Self {
        Self::Event(value)
    }
}
