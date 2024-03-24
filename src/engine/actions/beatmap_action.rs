use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum BeatmapAction {
    /// Play the currently selected beatmap and playmode
    PlaySelected,

    /// Set the current beatmap
    /// 
    /// map, use audio preview time, restart song?
    Set(Arc<BeatmapMeta>, SetBeatmapOptions),

    /// Set the current beatmap
    /// 
    /// map hash, use audio preview time, restart song?
    SetFromHash(Md5Hash, SetBeatmapOptions),

    /// Remove the current beatmap
    Remove,

    /// Set the current beatmap to the next map in the queue (or random if none exist)
    Next,

    /// Confirm the selected map
    /// This will play the map if in singleplayer mode, and select the map in multiplayer
    ConfirmSelected,

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

    /// Set the current playmode
    SetPlaymode(String),

    /// Perform an action on the list
    ListAction(BeatmapListAction),

    /// Add a beatmap
    AddBeatmap { map: Arc<BeatmapMeta>, add_to_db: bool },

    /// Let the beatmap manager know it has been initialized.
    /// This should only be used internally after loading the beatmaps from the db
    InitializeManager,
}
impl From<BeatmapAction> for TatakuAction {
    fn from(value: BeatmapAction) -> Self { Self::Beatmap(value) }
}

#[derive(Clone, Debug)]
pub enum PostDelete {
    Next,
    Previous,
    Random,
}


/// What to do if the desired action isnt possible
#[derive(Copy, Clone, Debug, Default)]
pub enum MapActionIfNone {
    /// Continue with the current map (ie dont change)
    #[default]
    ContinueCurrent,

    /// Remove the current beatmap
    SetNone,

    /// Set a random map
    /// 
    /// use preview time?
    Random(bool),
}


/// An action that affects the list of beatmaps
// TODO: add descriptions 
#[derive(Clone, Debug)]
pub enum BeatmapListAction {
    NextMap,
    PrevMap,
    NextSet,
    PrevSet,
    SelectSet(usize),

    Refresh {
        filter: Option<String>,
    },
}


#[derive(Copy, Clone, Debug, Default, ChainableInitializer)]
pub struct SetBeatmapOptions {
    #[chain]
    pub use_preview_point: bool,
    #[chain]
    pub restart_song: bool,
    #[chain]
    pub if_none: MapActionIfNone
}
impl SetBeatmapOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

