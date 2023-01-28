pub mod modes;

// pub mod diff_calc;
mod game_mode;
mod ingame_manager;
mod gameplay_helpers;

pub use game_mode::*;
pub use ingame_manager::*;
pub use gameplay_helpers::*;

use crate::prelude::*;

#[async_trait]
pub trait DiffCalc: Send + Sync {
    async fn new(g: &BeatmapMeta) -> TatakuResult<Self> where Self:Sized;
    async fn calc(&mut self, mods: &ModManager) -> TatakuResult<DiffCalcSummary>;
}
#[derive(Default, serde::Serialize)]
pub struct DiffCalcSummary {
    pub diff: f32,
    pub diffs: Vec<f32>,
    pub strains: HashMap<String, Vec<f32>>
}
impl DiffCalcSummary {
    pub fn save(&self, path: impl AsRef<Path>) -> TatakuResult {
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}


pub fn calc_acc(score: &Score) -> f64 {
    get_gamemode_info(&score.playmode)
        .map(|i|i.calc_acc(score))
        .unwrap_or_default()
        .normal_or(1.0)
}

pub fn gamemode_display_name(playmode: &String) -> &str {
    get_gamemode_info(playmode)
        .map(|i|i.display_name())
        .unwrap_or("Unknown")
}

pub async fn manager_from_playmode(playmode: PlayMode, beatmap: &BeatmapMeta) -> TatakuResult<IngameManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let playmode = beatmap.playmode(playmode);

    let info = get_gamemode_info(&playmode)
        .ok_or_else(||TatakuError::GameMode(GameModeError::UnknownGameMode))?;

    let gamemode = info.create_game(&beatmap).await?;

    Ok(IngameManager::new(beatmap, gamemode).await)
}


pub fn perfcalc_for_playmode(playmode: &String) -> PerformanceCalc {{
    get_gamemode_info(&playmode)
        .map(|i|i.get_perf_calc())
        .unwrap_or(Box::new(|diff, acc| {
            let perf = diff * (acc / 0.99).powi(6);
            #[cfg(feature="debug_perf_rating")]
            println!("diff:{diff}, acc: {acc} = perf {perf}");
            perf
        }))
}}


pub async fn calc_diff(map: &BeatmapMeta, mode_override: String) -> TatakuResult<Box<dyn DiffCalc>> {{
    let playmode = map.check_mode_override(mode_override);

    get_gamemode_info(&playmode)
        .ok_or_else(||TatakuError::GameMode(GameModeError::UnknownGameMode))?
        .create_diffcalc(map).await
}}

