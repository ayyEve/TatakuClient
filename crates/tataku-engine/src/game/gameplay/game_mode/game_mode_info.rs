use crate::prelude::*;
use futures_util::future::BoxFuture;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GameModeInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub about: &'static str,
    pub author: &'static str,
    pub author_contact: &'static str,
    pub bug_report_url: &'static str,

    pub mods: &'static [GameplayModGroupStatic],
    pub stat_groups: &'static [StatGroup],
    pub judgments: &'static [HitJudgment],
    pub diff_values: &'static [DifficultyValue],

    pub calc_acc: fn(&Score) -> f32,
    pub calc_perf: fn(CalcPerfInfo<'_>) -> f32,

    pub can_load_beatmap: fn(&BeatmapType) -> bool,

    // pub get_diff_string: fn(&BeatmapMetaWithDiff, &ModManager) -> String,
    pub stats_from_groups: fn(&HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo>,

    pub create_game: for<'a> fn(&'a Beatmap, &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn GameMode>>>,
    pub create_diffcalc: for<'a> fn(&'a BeatmapMeta, &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn DiffCalc>>>,
}
impl GameModeInfo {
    pub const DEFAULT: Self = Self {
        id: "none",
        display_name: "None",
        about: "",
        author: "",
        author_contact: "",
        bug_report_url: "",
        mods: &[],
        stat_groups: &[],
        judgments: &[],
        diff_values: &[],
        calc_acc: |_| 0.0,
        calc_perf: |_| 0.0,
        // get_diff_string: Self::dummy_diff_str,
        stats_from_groups: |_| Vec::new(),
        can_load_beatmap: |_| false,
        create_game: |_, _| Box::pin(async { Err(GameModeError::UnknownGameMode.into()) }),
        create_diffcalc: |_,_| Box::pin(async { Err(GameModeError::UnknownGameMode.into()) }),
    };

    // fn dummy_calc_acc(_:&Score) -> f32 { 0.0 }
    // fn dummy_calc_perf(_:CalcPerfInfo<'_>) -> f32 { 0.0 }
    // fn dummy_diff_str(_: &BeatmapMetaWithDiff, _: &ModManager) -> String { String::new() }
    // fn dummy_stats(_: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> { Vec::new() }
    // fn dummy_can_load_beatmap(_: &BeatmapType) -> bool { false }

    // fn dummy_create_game<'a>(_: &'a Beatmap, _: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn GameMode>>> { Box::pin(async { Err(GameModeError::UnknownGameMode.into()) }) }
    // fn dummy_create_diffcalc<'a>(_: &'a BeatmapMeta, _: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn DiffCalc>>> { Box::pin(async { Err(GameModeError::UnknownGameMode.into()) }) }

    pub fn calc_acc(&self, score: &Score) -> f32 {
        (self.calc_acc)(score)
    }
    pub fn calc_perf(&self, data: CalcPerfInfo<'_>) -> f32 {
        (self.calc_perf)(data)
    }

    // pub fn get_diff_string(&self, map: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
    //     (self.get_diff_string)(map, mods)
    // }
    pub fn stats_from_groups(&self, stats: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> {
        (self.stats_from_groups)(stats)
    }

    pub fn can_load_beatmap(&self, map: &BeatmapType) -> bool {
        (self.can_load_beatmap)(map)
    }



    pub async fn create_game(&self, map: &Beatmap, settings: &Settings) -> TatakuResult<Box<dyn GameMode>> {
        (self.create_game)(map, settings).await
    }
    
    pub async fn create_diffcalc(&self, map: &BeatmapMeta, settings: &Settings) -> TatakuResult<Box<dyn DiffCalc>> {
        (self.create_diffcalc)(map, settings).await
    }


}
impl Default for GameModeInfo {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl std::fmt::Debug for GameModeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameModeInfo")
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("about", &self.about)
            .field("author", &self.author)
            .field("author_contact", &self.author_contact)
            .field("bug_report_url", &self.bug_report_url)
            .field("mods", &self.mods)
            .field("stat_groups", &self.stat_groups)
            .field("judgments", &self.judgments)
            // .field("calc_acc", &self.calc_acc)
            // .field("calc_perf", &self.calc_perf)
            // .field("get_diff_string", &self.get_diff_string)
            // .field("stats_from_groups", &self.stats_from_groups)
            // .field("create_game", &self.create_game)
            // .field("create_diffcalc", &self.create_diffcalc)
            .finish()
    }
}


pub struct CalcPerfInfo<'a> {
    pub score: &'a Score,
    pub accuracy: f32,
    pub map_difficulty: f32,
}

// const TEST: GameModeInfoTest = GameModeInfoTest {
//     id: "",
//     display_name: "",
//     about: "",
//     author: "",
//     author_contact: "",
//     bug_report_url: "",

//     calc_acc: &calc_acc_test,

// };

// fn calc_acc_test(score: &Score) -> f32 {
//     0.0
// }