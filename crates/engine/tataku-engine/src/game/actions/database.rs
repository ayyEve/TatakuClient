use crate::*;
use common::Md5Hash;
use engine::data::{
    BeatmapPreferences,
    BeatmapPlaymodePreferences,
};

#[derive(Clone, Debug)]
pub enum Action {
    ClearAllBeatmaps,
    AddBeatmaps(Vec<Arc<BeatmapMeta>>),

    AddScore(Box<common::Score>),

    SaveBeatmapPreferences {
        hash: Md5Hash,
        prefs: BeatmapPreferences,
    },
    SaveBeatmapPlaymodePreferences {
        hash: Md5Hash,
        playmode: ArcStr,
        prefs: BeatmapPlaymodePreferences,
    },

    /// ignore a beatmap
    IgnoreBeatmap(engine::database::IgnoredBeatmap),
}
impl From<Action> for engine::actions::Action {
    fn from(value: Action) -> Self {
        Self::Database(value)
    }
}
