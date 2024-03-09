use crate::prelude::*;

/// An action from a menu (or dialog) to tell the game to do something
pub enum MenuAction {
    /// Don't do anything (this is a helper)
    None,

    /// Perform a menu operation
    Menu(MenuMenuAction),

    /// Perform a game operation
    Game(GameMenuAction),

    /// Perform a beatmap operation
    Beatmap(BeatmapMenuAction),

    /// Perform an operation on the current song
    Song(SongMenuAction),

    /// Perform a widget operation
    PerformOperation(IcedOperation),

    /// Handle a multiplayer action
    MultiplayerAction(MultiplayerManagerAction),

    /// Quit the game
    Quit
}

impl MenuAction {
    pub fn set_menu(menu: impl AsyncMenu + 'static) -> Self {
        Self::Menu(MenuMenuAction::SetMenu(Box::new(menu)))
    }
}

impl From<MenuMenuAction> for MenuAction {
    fn from(value: MenuMenuAction) -> Self { Self::Menu(value) }
}
impl From<GameMenuAction> for MenuAction {
    fn from(value: GameMenuAction) -> Self { Self::Game(value) }
}
impl From<BeatmapMenuAction> for MenuAction {
    fn from(value: BeatmapMenuAction) -> Self { Self::Beatmap(value) }
}



pub enum MenuMenuAction {
    /// Set the current menu
    SetMenu(Box<dyn AsyncMenu>),

    /// Set the menu to a custom menu with the provided identifier
    SetMenuCustom(String),

    /// Go to the previous menu
    /// 
    /// This is mainly a helper fn for spec and multi
    PreviousMenu(&'static str),

    /// Add a dialog
    /// 
    /// dialog, allow_duplicates
    AddDialog(Box<dyn Dialog>, bool),

    /// Set the menu to a custom menu with the provided identifier
    AddDialogCustom(String, bool),
}

pub enum GameMenuAction {
    /// Start a game with the provided ingame manager
    StartGame(Box<IngameManager>),

    /// Resume a map
    ResumeMap(Box<IngameManager>),

    /// Watch a replay
    WatchReplay(Box<Replay>),
}


#[derive(Clone, Debug)]
pub enum BeatmapMenuAction {

    /// Play a map
    /// 
    /// map, playmode
    PlayMap(Arc<BeatmapMeta>, String),

    /// Play the currently selected beatmap and playmode
    PlaySelected,

    /// Set the current beatmap
    /// 
    /// map, use audio preview time
    Set(Arc<BeatmapMeta>, bool),

    /// Remove the current beatmap
    Remove,

    /// Set the current beatmap to the next map in the queue (or random if none exist)
    Next,

    /// Set the current beatmap to a random map
    /// 
    /// You probably want to use Next though
    /// 
    /// use preview point?
    Random(bool),

    /// Set the current beatmap to the next map in the queue (or do nothing if none exist)
    Previous(MapActionIfNone),

    /// Delete the provided beatmap
    Delete(Md5Hash),

    /// Delete the current beatmap
    DeleteCurrent(PostDelete),
}

#[derive(Clone, Debug)]
pub enum PostDelete {
    Next,
    Previous,
    Random,
}


/// What to do if the desired action isnt possible
#[derive(Clone, Debug)]
pub enum MapActionIfNone {
    /// Continue with the current map (ie dont change)
    ContinueCurrent,

    /// Set a random map
    /// 
    /// use preview time?
    Random(bool),
}


#[derive(Clone, Debug)]
pub enum SongMenuAction {
    /// Play/Resume the current song
    Play,

    /// Restart the current song
    Restart,

    /// Pause the current song
    Pause,

    /// Stop the current song
    Stop,

    /// Play/pause the current song
    Toggle,

    /// Seek by the specified amount (negative means seek backwards)
    SeekBy(f32),

    /// Set the position of the current song (in ms)
    SetPosition(f32),
}