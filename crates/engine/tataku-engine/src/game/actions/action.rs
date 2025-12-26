use crate::*;
use common::reflect::Reflect;

pub type ActionQueue = Vec<actions::Action>;

/// An action from a menu (or dialog) to tell the game to do something
#[derive(Debug2)]
pub enum Action {
    /// A delayed action
    #[debug(skip)]
    Delayed(DelayedActionType, u64),

    /// Perform an audio action
    Audio(actions::audio::AudioAction),

    /// Perform a menu operation
    #[cfg(feature="graphics")]
    Menu(actions::menu::MenuAction),

    /// Perform a game operation
    Game(Box<actions::game::GameAction>),

    /// Perform a game operation
    Online(actions::online::OnlineAction),

    /// Perform a beatmap operation
    Beatmap(actions::beatmap::BeatmapAction),

    /// Perform an operation on the current song
    Song(actions::song::SongAction),

    /// Perform a mods action
    Mods(actions::mods::ModAction),

    /// Perform an action on the Ui
    #[cfg(feature="graphics")]
    Ui(actions::ui::UiAction),

    /// Perform a multiplayer action
    Multiplayer(actions::multiplayer::MultiplayerAction),

    /// Perform a task action
    Task(actions::task::TaskAction),

    /// Perform a database operation
    Database(actions::database::Action),

    /// Perform a cursor action
    #[cfg(feature="graphics")]
    CursorAction(actions::cursor::CursorAction),

    /// Perform a window action
    #[cfg(feature="graphics")]
    WindowAction(Box<actions::window::WindowAction>),

    /// Download a file
    Download(#[debug(skip)] Box<engine::io::Downloadable>),

    /// Perform an online content action
    OnlineContent(actions::online_content::OnlineContentAction),

    /// Handle an event
    Event(Box<engine::TatakuIntegrationEvent>),

    /// Handle multiple actions
    Multiple(Vec<Self>),
}
// impl Clone for actions::Action {
//     fn clone(&self) -> Self {
//         match self {
//             Self::None => Self::None,

//             Self::Delayed(a, time)
//                 => Self::Delayed(a.clone(), *time),

//             Self::Audio(a) => Self::Audio(a.clone()),
//             Self::Game(a) => Self::Game(a.clone()),
//             Self::Online(a) => Self::Online(a.clone()),
//             Self::Beatmap(a) => Self::Beatmap(a.clone()),
//             Self::Song(a) => Self::Song(a.clone()),
//             Self::Mods(a) => Self::Mods(a.clone()),
//             Self::Multiplayer(a) => Self::Multiplayer(a.clone()),
//             Self::Download(a) => Self::Download(a.clone()),
//             Self::OnlineContent(a) => Self::OnlineContent(a.clone()),
//             Self::Event(a) => Self::Event(a.clone()),
//             Self::Multiple(a) => Self::Multiple(a.clone()),
            
//             #[cfg(feature="graphics")]
//             Self::Menu(a) => Self::Menu(a.clone()),
//             #[cfg(feature="graphics")]
//             Self::CursorAction(a) => Self::CursorAction(*a),
//             #[cfg(feature="graphics")]
//             Self::WindowAction(a) => Self::WindowAction(a.clone()),
            

//             #[cfg(feature="graphics")]
//             Self::Ui(_) => panic!("trying to clone UiAction"),
//             Self::Task(_) => panic!("trying to clone TaskAction"),
//         }
//     }
// }



impl<T:engine::Task + 'static> From<T> for actions::Action {
    fn from(value: T) -> Self {
        Self::Task(actions::task::TaskAction::AddTask(Box::new(value)))
    }
}

impl From<engine::Notification> for actions::Action {
    fn from(value: engine::Notification) -> Self {
        Self::Game(Box::new(actions::game::GameAction::AddNotification(value)))
    }
}

impl From<engine::TatakuIntegrationEvent> for actions::Action {
    fn from(value: engine::TatakuIntegrationEvent) -> Self {
        Self::Event(Box::new(value))
    }
}

impl From<engine::Downloadable> for actions::Action {
    fn from(value: engine::Downloadable) -> Self {
        Self::Download(Box::new(value))
    }
}

pub enum DelayedActionType {
    Action(Box<actions::Action>),
    Callback(Arc<dyn Fn(&mut dyn Reflect) -> actions::Action + Send + Sync>)
}
