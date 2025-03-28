use crate::prelude::*;

pub enum GameAction {
    /// Fully quit the game
    Quit,

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

    RestartOnline,

    /// Handle an event
    #[cfg(feature="graphics")]
    HandleEvent(TatakuEventType, Option<TatakuValue>),

    /// Add a notification
    AddNotification(Notification),

    /// Update the game's background
    UpdateBackground,

    /// Copy some text to the clipboard
    CopyToClipboard(String),

    /// Force a refresh of global.playmode and global.playmode_actual (+display) variables
    RefreshPlaymodeValues,

    /// Set the actual playmode for the current beatmap
    UpdatePlaymodeActual(String),

    #[cfg(feature="graphics")]
    NewGameplayManager(NewManager),
    DropGameplayManager(GameplayId),
    GameplayAction(GameplayId, GameplayAction),
    CurrentGameAction(CurrentGameAction),

    /// update settings with the provided callback
    UpdateSettings(Box<dyn FnOnce(&mut Settings) + Send + Sync>),
}

impl From<GameAction> for TatakuAction {
    fn from(value: GameAction) -> Self { Self::Game(Box::new(value)) }
}

impl core::fmt::Debug for GameAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quit => write!(f, "Quit"),
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
            
            Self::RestartOnline => write!(f, "RestartOnline"),

            #[cfg(feature="graphics")]
            Self::NewGameplayManager(arg0) => f.debug_tuple("NewGameplayManager").field(arg0).finish(),
            Self::DropGameplayManager(arg0) => f.debug_tuple("DropGameplayManager").field(arg0).finish(),
            Self::GameplayAction(arg0, arg1) => f.debug_tuple("GameplayAction").field(arg0).field(arg1).finish(),
            Self::RefreshPlaymodeValues => write!(f, "RefreshPlaymodeValues"),
            Self::UpdatePlaymodeActual(arg0) => f.debug_tuple("UpdatePlaymodeActual").field(arg0).finish(),
            Self::UpdateSettings(_)=> write!(f, "UpdateSettings"),

            Self::CurrentGameAction(action) => f.debug_tuple("CurrentGameAction").field(action).finish(),

            // Self::ForceUiRefresh => write!(f, "ForceUiRefresh"),
            // Self::UiNodeDirty(_) => write!(f, "UiNodeDirty"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CurrentGameAction {
    /// Start whatever game is saved
    Start,

    /// Resume a game
    Resume,

    /// Pause the current game and open the provided menu
    Pause(String),

    Restart,

    Free,
}
impl From<CurrentGameAction> for TatakuAction {
    fn from(value: CurrentGameAction) -> Self {
        Self::Game(Box::new(GameAction::CurrentGameAction(value)))
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
    Replay(Box<Score>),
    Spectator(Box<SpectatorGameplayInfo>),
}
#[derive(Debug, Clone, Default)]
pub struct SpectatorGameplayInfo {
    pub host_id: u32,
    pub host_username: String,

    pub pending_frames: VecDeque<SpectatorFrame>,
    pub spectators: HashMap<u32, String>,
}

