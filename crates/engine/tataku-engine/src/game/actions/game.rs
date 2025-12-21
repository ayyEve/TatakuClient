use crate::*;
use common::Score;
use input::TatakuEvent;
use tataku::TatakuValue;
use common::types::network::spectator::SpectatorFrame;
pub type GameplayId = Arc<u32>;

#[derive(Debug2)]
pub enum GameAction {
    /// Fully quit the game
    Quit,

    /// Watch a replay
    WatchReplay(Box<Score>),

    /// Update a value
    SetValue(String, TatakuValue),

    /// Open a score in the score menu
    #[cfg(feature="graphics")]
    ViewScore(gameplay::IngameScore),

    /// Open a score in the score menu
    #[cfg(feature="graphics")]
    ViewScoreId(usize),

    /// Handle a message
    #[cfg(feature="graphics")]
    HandleMessage(ui::message::Message),

    /// Refresh the scores list
    RefreshScores,

    /// Reload the online manager
    RestartOnline,

    /// Handle an event
    HandleEvent(input::TatakuEvent, Option<TatakuValue>),

    /// Add a notification
    AddNotification(engine::Notification),

    /// Update the game's background
    UpdateBackground,

    /// Copy some text to the clipboard
    CopyToClipboard(ArcStr),

    /// Force a refresh of global.playmode and global.playmode_actual (+display) variables
    RefreshPlaymodeValues,

    /// Force a refresh of the skins list
    RefreshSkins,

    /// Set the actual playmode for the current beatmap
    UpdatePlaymodeActual(ArcStr),

    #[cfg(feature="graphics")]
    NewGameplayManager(NewManager),
    DropGameplayManager(GameplayId),
    GameplayAction(GameplayId, actions::gameplay::GameplayAction),
    CurrentGameAction(CurrentGameAction),

    /// update settings with the provided callback
    UpdateSettings(#[debug(skip)] Arc<dyn Fn(&mut engine::Settings) + Send + Sync>),
}

impl From<GameAction> for actions::Action {
    fn from(value: GameAction) -> Self {
        Self::Game(Box::new(value))
    }
}

impl From<TatakuEvent> for actions::Action {
    fn from(value: TatakuEvent) -> Self {
        GameAction::HandleEvent(value, None).into()
    }
}
impl From<(TatakuEvent, TatakuValue)> for actions::Action {
    fn from(value: (TatakuEvent, TatakuValue)) -> Self {
        GameAction::HandleEvent(value.0, Some(value.1)).into()
    }
}


#[derive(Clone, Debug)]
pub enum CurrentGameAction {
    /// Start whatever game is saved
    Start,

    /// Resume a game
    Resume,

    /// Pause the current game and open the provided menu
    Pause {
        id: String,
    },

    Restart,

    Free,
}
impl From<CurrentGameAction> for actions::Action {
    fn from(value: CurrentGameAction) -> Self {
        Self::Game(Box::new(GameAction::CurrentGameAction(value)))
    }
}


#[cfg(feature="graphics")]
#[derive(Default, Clone, Debug2)]
pub struct NewManager {
    /// who is requesting the manager?
    pub owner: ui::message::MessageSource,
    /// what mods should be used? if none, will use the global mods (and will update mods when global mods update)
    pub mods: Option<gameplay::mods::ModManager>,
    /// what map hash to use
    pub map_hash: Option<common::Md5Hash>,
    /// optional path to the map hash
    pub path: Option<ArcStr>,
    /// what playmode to use. if none, will use
    pub playmode: Option<ArcStr>,
    /// what gameplay mode to use.
    pub gameplay_mode: Option<GameplayTypeInfo>,
    /// if it should be bound to an area
    pub area: Option<tataku::Bounds>,
    /// if there is a different draw function that should be used (mainly for widgets)
    #[debug(skip)]
    pub draw_function: Option<Arc<dyn Fn(graphics::RenderableCollection) + Send + Sync + 'static>>,
}

#[derive(Debug, Clone, Default)]
pub enum GameplayTypeInfo {
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
    pub host_username: ArcStr,

    pub pending_frames: VecDeque<SpectatorFrame>,
    pub spectators: HashMap<u32, ArcStr>,
}
