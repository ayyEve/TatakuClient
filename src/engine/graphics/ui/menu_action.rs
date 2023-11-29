use crate::prelude::*;

/// An action from a menu (or dialog) to tell the game to do something
pub enum MenuAction {
    /// Set the current menu
    SetMenu(Box<dyn AsyncMenu>),

    /// Go to the previous menu
    /// 
    /// This is mainly a helper fn for spec and multi
    PreviousMenu(&'static str),

    /// Add a dialog
    /// 
    /// dialog, allow_duplicates
    AddDialog(Box<dyn Dialog>, bool),

    /// Set the current beatmap
    /// 
    /// map, use audio preview time
    SetBeatmap(Arc<BeatmapMeta>, bool),

    /// Remove the current beatmap
    RemoveBeatmap,

    /// Delete the provided beatmap
    DeleteBeatmap(Md5Hash),

    /// Watch a replay
    WatchReplay(Box<Replay>),

    /// Play a map
    /// 
    /// map, playmode
    PlayMap(Arc<BeatmapMeta>, String),

    /// Resume a map
    ResumeMap(Box<IngameManager>),

    /// Start a game with the provided ingame manager
    StartGame(Box<IngameManager>),

    /// Perform a widget operation
    PerformOperation(IcedOperation),

    /// Handle a multiplayer action
    MultiplayerAction(MultiplayerManagerAction),

    /// Quit the game
    Quit
}

