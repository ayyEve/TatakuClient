use crate::prelude::*;

/// An action from a menu (or dialog) to tell the game to do something
#[derive(Default, Debug2)]
pub enum TatakuAction {
    /// Don't do anything (this is a helper)
    #[default] None,

    /// A delayed action
    #[debug(skip)]
    Delayed(DelayedActionType, u64),

    /// Perform an audio action
    Audio(AudioAction),

    /// Perform a menu operation
    #[cfg(feature="graphics")]
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
    #[cfg(feature="graphics")]
    Ui(UiAction),

    /// Perform a multiplayer action
    Multiplayer(MultiplayerAction),

    /// Perform a task action
    Task(TaskAction),

    /// Perform a cursor action
    #[cfg(feature="graphics")]
    CursorAction(CursorAction),

    /// Perform a window action
    #[cfg(feature="graphics")]
    WindowAction(WindowAction),

    /// Download a file
    Download(#[debug(skip)] Box<Downloadable>),

    /// Handle an event
    Event(TatakuIntegrationEvent),

    /// Handle multiple actions
    Multiple(Vec<Self>)
}

impl<T:TatakuTask + 'static> From<T> for TatakuAction {
    fn from(value: T) -> Self {
        Self::Task(TaskAction::AddTask(Box::new(value)))
    }
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


pub enum DelayedActionType {
    Action(Box<TatakuAction>),
    Callback(Box<dyn FnOnce(&mut dyn Reflect) -> TatakuAction + Send + Sync>)
}
