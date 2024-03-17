use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum BeatmapAction {

    /// Play a map
    /// 
    /// map, playmode
    PlayMap(Arc<BeatmapMeta>, String),

    /// Play the currently selected beatmap and playmode
    PlaySelected,

    /// Set the current beatmap
    /// 
    /// map, use audio preview time, restart song?
    Set(Arc<BeatmapMeta>, bool, bool),

    /// Set the current beatmap
    /// 
    /// map hash, use audio preview time, restart song?
    SetFromHash(Md5Hash, bool, bool),

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

    /// perform an action on the list
    ListAction(BeatmapListAction),
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
#[derive(Clone, Debug)]
pub enum MapActionIfNone {
    /// Continue with the current map (ie dont change)
    ContinueCurrent,

    /// Set a random map
    /// 
    /// use preview time?
    Random(bool),
}


/// An action that affects the list of beatmaps
#[derive(Clone, Debug)]
pub enum BeatmapListAction {
    Refresh {
        filter: Option<String>,
    },
}
