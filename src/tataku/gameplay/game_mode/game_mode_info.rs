use crate::prelude::*;

#[async_trait]
pub trait GameModeInfo {
    fn new() -> Self where Self:Sized;
    fn display_name(&self) -> &str;
    fn about(&self) -> &str { "No description" }
    fn author(&self) -> &str { "No author?" }
    fn author_contact(&self) -> &str { "No author contact" }
    fn bug_report_url(&self) -> &str { "No bug report url" }

    
    fn calc_acc(&self, score: &Score) -> f64;
    fn get_perf_calc(&self) -> PerformanceCalc {
        Box::new(|diff, acc| {
            let perf = diff * (acc / 0.99).powi(6);
            #[cfg(feature="debug_perf_rating")]
            println!("diff:{diff}, acc: {acc} = perf {perf}");
            perf
        })
    }
    fn get_mods(&self) -> Vec<GameplayModGroup> { Vec::new() }
    fn get_stat_groups(&self) -> Vec<StatGroup> { Vec::new() }
    fn get_judgments(&self) -> Box<dyn crate::prelude::HitJudgments>;
    fn get_diff_string(&self, info: &BeatmapMetaWithDiff, mods: &ModManager) -> String;
    
    async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>>;
    async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>>;

    fn stats_from_groups(&self, _data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> { Vec::new() }
}