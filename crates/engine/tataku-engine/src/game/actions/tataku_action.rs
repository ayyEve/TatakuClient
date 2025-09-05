use crate::prelude::*;

pub type ActionQueue = Vec<TatakuAction>;

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
    WindowAction(Box<WindowAction>),

    /// Download a file
    Download(#[debug(skip)] Box<Downloadable>),

    /// Perform an online content action
    OnlineContent(OnlineContentAction),

    /// Handle an event
    Event(Box<TatakuIntegrationEvent>),

    /// Handle multiple actions
    Multiple(Vec<Self>),
}
impl Clone for TatakuAction {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,

            Self::Delayed(a, time) 
                => Self::Delayed(a.clone(), *time),

            Self::Audio(a) => Self::Audio(a.clone()),
            Self::Game(a) => Self::Game(a.clone()),
            Self::Online(a) => Self::Online(a.clone()),
            Self::Beatmap(a) => Self::Beatmap(a.clone()),
            Self::Song(a) => Self::Song(a.clone()),
            Self::Mods(a) => Self::Mods(a.clone()),
            Self::Multiplayer(a) => Self::Multiplayer(a.clone()),
            Self::Download(a) => Self::Download(a.clone()),
            Self::OnlineContent(a) => Self::OnlineContent(a.clone()),
            Self::Event(a) => Self::Event(a.clone()),
            Self::Multiple(a) => Self::Multiple(a.clone()),
            
            #[cfg(feature="graphics")]
            Self::Menu(a) => Self::Menu(a.clone()),
            #[cfg(feature="graphics")]
            Self::Ui(a) => Self::Ui(a.clone()),
            #[cfg(feature="graphics")]
            Self::CursorAction(a) => Self::CursorAction(*a),
            #[cfg(feature="graphics")]
            Self::WindowAction(a) => Self::WindowAction(a.clone()),

            
            Self::Task(_) => panic!("trying to clone TaskAction"),
        }
    }
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
        Self::Event(Box::new(value))
    }
}

impl From<Downloadable> for TatakuAction {
    fn from(value: Downloadable) -> Self {
        Self::Download(Box::new(value))
    }
}

#[derive(Clone)]
pub enum DelayedActionType {
    Action(Box<TatakuAction>),
    Callback(Arc<dyn Fn(&mut dyn Reflect) -> TatakuAction + Send + Sync>)
}
