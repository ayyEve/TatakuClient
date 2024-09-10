use crate::prelude::*;

pub enum GameAction {
    /// Fully quit the game
    Quit,

    /// Start a game with the provided ingame manager
    StartGame(Box<GameplayManager>),

    /// Resume a map
    ResumeMap(Box<GameplayManager>),

    /// Watch a replay
    WatchReplay(Box<Score>),

    /// Update a value 
    SetValue(String, TatakuValue),

    /// Open a score in the score menu
    ViewScore(IngameScore),

    /// Open a score in the score menu
    ViewScoreId(usize),

    /// Handle a message
    #[cfg(feature="graphics")]
    HandleMessage(Message),

    /// Refresh the scores list
    RefreshScores,

    /// Handle an event
    #[cfg(feature="graphics")]
    HandleEvent(TatakuEventType, Option<TatakuValue>),

    /// Add a notification
    AddNotification(Notification),

    /// Update the game's background
    UpdateBackground,

    /// Copy some text to the clipboard
    CopyToClipboard(String),

    /// Force a refresh of the ui, ie if the values map changed
    ForceUiRefresh,

    /// Force a refresh of global.playmode and global.playmode_actual (+display) variables
    RefreshPlaymodeValues,

    /// Set the actual playmode for the current beatmap
    UpdatePlaymodeActual(String),

    ///
    #[cfg(feature="graphics")]
    NewGameplayManager(NewManager),
    DropGameplayManager(GameplayId),
    GameplayAction(GameplayId, GameplayAction),

    /// free up an existing gameplay manager (clean up its textures)
    FreeGameplay(Box<GameplayManager>),
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
            #[cfg(feature="graphics")]
            Self::HandleMessage(arg0) => f.debug_tuple("HandleMessage").field(arg0).finish(),
            Self::RefreshScores => write!(f, "RefreshScores"),
            #[cfg(feature="graphics")]
            Self::HandleEvent(arg0, arg1) => f.debug_tuple("HandleEvent").field(arg0).field(arg1).finish(),
            Self::AddNotification(arg0) => f.debug_tuple("AddNotification").field(arg0).finish(),
            Self::UpdateBackground => write!(f, "UpdateBackground"),
            Self::CopyToClipboard(arg0) => f.debug_tuple("CopyToClipboard").field(arg0).finish(),
            #[cfg(feature="graphics")]
            Self::NewGameplayManager(arg0) => f.debug_tuple("NewGameplayManager").field(arg0).finish(),
            Self::DropGameplayManager(arg0) => f.debug_tuple("DropGameplayManager").field(arg0).finish(),
            Self::GameplayAction(arg0, arg1) => f.debug_tuple("GameplayAction").field(arg0).field(arg1).finish(),
            Self::FreeGameplay(_) => write!(f, "FreeGameplay"),
            Self::ForceUiRefresh => write!(f, "ForceUiRefresh"),
            Self::RefreshPlaymodeValues => write!(f, "RefreshPlaymodeValues"),
            Self::UpdatePlaymodeActual(arg0) => f.debug_tuple("UpdatePlaymodeActual").field(arg0).finish(),
        }
    }
}
pub type GameplayId = Arc<u32>;


#[cfg(feature="graphics")]
#[derive(Default, Clone)]
pub struct NewManager {
    /// who is requesting the manager?
    pub owner: MessageOwner,
    /// what mods should be used? if none, will use the global mods (and will update mods when global mods update)
    pub mods: Option<ModManager>,
    /// what map hash to use
    pub map_hash: Option<Md5Hash>,
    /// optional path to the map hash 
    pub path: Option<String>,
    /// what playmode to use. if none, will use 
    pub playmode: Option<String>,
    /// what gameplay mode to use.
    pub gameplay_mode: Option<GameplayMode>,
    /// if it should be bound to an area
    pub area: Option<Bounds>,
    /// if there is a different draw function that should be used (mainly for widgets)
    pub draw_function: Option<Arc<dyn Fn(TransformGroup) + Send + Sync + 'static>>,
}
#[cfg(feature="graphics")]
impl std::fmt::Debug for NewManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewManager")
            .field("owner", &self.owner)
            .field("mods", &self.mods)
            .field("map_hash", &self.map_hash)
            .field("path", &self.path)
            .field("playmode", &self.playmode)
            .field("gameplay_mode", &self.gameplay_mode)
            .field("area", &self.area)
            .field("draw_function", &self.draw_function.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Default)]
pub enum GameplayMode {
    #[default]
    Normal,
    Preview,
    Multiplayer,
    Replay(Score),
    Spectator {
        host_id: u32,
        host_username: String,

        pending_frames: VecDeque<SpectatorFrame>,
        spectators: HashMap<u32, String>,
    }
}