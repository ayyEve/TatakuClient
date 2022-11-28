use crate::prelude::*;

#[async_trait]
pub trait GameModeInfo {
    fn new() -> Self where Self:Sized;
    fn display_name(&self) -> &str;

    
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
    fn get_judgments(&self) -> Box<dyn crate::prelude::HitJudgments>;
    fn get_diff_string(&self, info: &BeatmapMetaWithDiff, mods: &ModManager) -> String;
    
    async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>>;
    async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>>;
}