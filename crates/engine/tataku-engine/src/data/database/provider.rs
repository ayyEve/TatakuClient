use crate::*;
use engine::data;
use common::Md5Hash;

pub trait DatabaseProvider: BeatmapProvider + BeatmapPreferencesProvider + ScoreProvider {}

pub trait BeatmapPreferencesProvider {
    // beatmap prefs
    fn get_beatmap_preferences(
        &self,
        map: Md5Hash,
    ) -> tataku::Result<data::BeatmapPreferences>;
    fn set_beatmap_preferences(
        &mut self,
        map: Md5Hash,
        prefs: &data::BeatmapPreferences,
    ) -> tataku::Result<()>;

    // beatmap playmode prefs
    fn get_beatmap_playmode_preferences(
        &self,
        map: Md5Hash,
        playmode: &str,
    ) -> tataku::Result<data::BeatmapPlaymodePreferences>;
    fn set_beatmap_playmode_preferences(
        &mut self,
        map: Md5Hash,
        playmode: &str,
        prefs: &data::BeatmapPlaymodePreferences
    ) -> tataku::Result<()>;
}

pub trait BeatmapProvider {
    // beatmaps
    fn get_beatmaps(&self) -> tataku::Result<Vec<Arc<engine::BeatmapMeta>>>;
    fn add_beatmaps(&mut self, maps: &[Arc<engine::BeatmapMeta>]) -> tataku::Result<()>;
    fn clear_all_beatmaps(&mut self) -> tataku::Result<()>;

    // ignored beatmaps
    fn get_ignored_beatmaps(&self) -> tataku::Result<Vec<data::IgnoredBeatmap>>;
    fn add_ignored_beatmap(&mut self, ignored: &data::IgnoredBeatmap) -> tataku::Result<()>;
    fn remove_ignored_beatmap(&mut self, ignored: &data::IgnoredBeatmap) -> tataku::Result<()>;

    // beatmap collections
    fn get_beatmap_collections(&self) -> tataku::Result<Vec<data::BeatmapCollection>>;
    fn update_beatmap_collection(&mut self, collection: &data::BeatmapCollection) -> tataku::Result<()>;
    fn add_beatmap_collection(&mut self, collection: &data::BeatmapCollection) -> tataku::Result<()>;
    fn remove_beatmap_collection(&mut self, name: &str) -> tataku::Result<()>;
}

pub trait DifficultyProvider: Send + Sync {
    fn get_diff(
        &mut self,
        map: &Arc<BeatmapMeta>,
        playmode: &str,
        mods: &engine::gameplay::Mods
    ) -> tataku::Result<f32>;
}

pub trait ScoreProvider {
    fn get_scores(
        &self,
        map: Md5Hash,
        playmode: &str,
        infos: &engine::gameplay::GamemodeInfos,
    ) -> tataku::Result<Vec<common::Score>>;

    fn add_score(&mut self, score: &common::Score) -> tataku::Result<()>;
}
