pub mod modes;

// pub mod diff_calc;
mod game_mode;
mod gameplay_manager;
mod gameplay_helpers;

pub use game_mode::*;
pub use gameplay_manager::*;
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
    #[allow(unused)]
    pub fn save(&self, path: impl AsRef<Path>) -> TatakuResult {
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}


pub fn calc_acc(score: &Score) -> f32 {
    get_gamemode_info(&score.playmode)
        .map(|i| i.calc_acc(score))
        .unwrap_or_default()
        .normal_or(1.0)
        as f32
}

pub fn gamemode_display_name(playmode: &str) -> &'static str {
    get_gamemode_info(playmode)
        .map(|i| i.display_name())
        .unwrap_or("Unknown")
}


pub async fn manager_from_playmode_path_hash<'a>(
    playmode: impl ToString, 
    map_path: String, 
    map_hash: Md5Hash,
    mods: ModManager,
) -> TatakuResult<GameplayManager> {
    let playmode = playmode.to_string();
    let beatmap = Beatmap::from_path_and_hash(map_path, map_hash)?;
    let playmode = beatmap.playmode(playmode.clone());

    let info = get_gamemode_info(&playmode)
        .ok_or(TatakuError::GameMode(GameModeError::UnknownGameMode))?;

    let gamemode = info.create_game(&beatmap).await?;

    Ok(GameplayManager::new(beatmap, gamemode, mods).await)
}

pub async fn manager_from_playmode(
    playmode: String, 
    beatmap: &BeatmapMeta,
    mods: ModManager,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let playmode = beatmap.playmode(playmode);

    let info = get_gamemode_info(&playmode)
        .ok_or(TatakuError::GameMode(GameModeError::UnknownGameMode))?;

    let gamemode = info.create_game(&beatmap).await?;

    Ok(GameplayManager::new(beatmap, gamemode, mods).await)
}


pub fn perfcalc_for_playmode(playmode: &str) -> PerformanceCalc {{
    get_gamemode_info(playmode)
        .map(|i| i.get_perf_calc())
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
        .ok_or_else(|| TatakuError::GameMode(GameModeError::UnknownGameMode))?
        .create_diffcalc(map).await
}}

