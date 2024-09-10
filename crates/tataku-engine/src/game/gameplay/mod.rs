// pub mod diff_calc;
mod stats;
mod game_mode;
mod gamemode;
mod gameplay_mods;
mod gameplay_manager;
mod gameplay_helpers;

pub use stats::*;
pub use gamemode::*;
pub use game_mode::*;
pub use gameplay_mods::*;
pub use gameplay_manager::*;
pub use gameplay_helpers::*;

use crate::prelude::*;

#[async_trait]
pub trait DiffCalc: Send + Sync {
    async fn new(g: &BeatmapMeta, settings: &Settings) -> TatakuResult<Self> where Self:Sized;
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


// pub fn calc_acc(score: &Score) -> f32 {
//     get_gamemode_info(&score.playmode)
//         .map(|i| i.calc_acc(score))
//         .unwrap_or_default()
//         .normal_or(1.0)
// }

// pub fn gamemode_display_name(playmode: &str) -> &'static str {
//     get_gamemode_info(playmode)
//         .map(|i| i.display_name())
//         .unwrap_or("Unknown")
// }
#[derive(Default, Debug, Clone)]
pub struct GamemodeInfos {
    pub by_id: Arc<HashMap<&'static str, GameModeInfo>>,
    pub by_num: Arc<Vec<GameModeInfo>>,

    _libraries: Arc<Vec<libloading::Library>>,
}
impl GamemodeInfos {
    pub fn new(list: Vec<GamemodeLibrary>) -> Self {

        let (libraries, by_num): (_, Vec<GameModeInfo>) = list.into_iter()
            .map(|i| (i._lib, i.info))
            .unzip();

        Self {
            by_id: Arc::new(by_num.iter()
                .map(|i| (i.id, *i))
                .collect()
            ),
            by_num: Arc::new(by_num),
            _libraries: Arc::new(libraries),
        }
    }
    pub fn get_info(&self, gamemode: &str) -> TatakuResult<&GameModeInfo> {
        self.by_id
            .get(gamemode)
            .ok_or(TatakuError::GameMode(GameModeError::UnknownGameMode))
    }
}

pub async fn manager_from_playmode_path_hash<'a>(
    infos: &GamemodeInfos,
    incoming_mode: &str,
    map_path: String,
    map_hash: Md5Hash,
    mods: ModManager,
    settings: &Settings,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_path_and_hash(map_path, map_hash)?;
    let playmode = beatmap.playmode(incoming_mode.to_owned());

    let info = infos.get_info(&playmode)?;

    let gamemode = info.create_game(&beatmap, settings).await?;
    Ok(GameplayManager::new(beatmap, gamemode, mods, settings).await)
}

pub async fn manager_from_playmode(
    infos: &GamemodeInfos,
    incoming_mode: &str,
    beatmap: &BeatmapMeta,
    mods: ModManager,
    settings: &Settings,
) -> TatakuResult<GameplayManager> {
    let beatmap = Beatmap::from_metadata(beatmap)?;
    let playmode = beatmap.playmode(incoming_mode.to_owned());

    let info = infos.get_info(&playmode)?;

    let gamemode = info.create_game(&beatmap, settings).await?;

    Ok(GameplayManager::new(beatmap, gamemode, mods, settings).await)
}


// pub fn perfcalc_for_playmode(
//     playmode: &str
// ) -> PerformanceCalc {{
//     get_gamemode_info(playmode)
//         .map(|i| i.get_perf_calc())
//         .unwrap_or(Box::new(|diff, acc| {
//             let perf = diff * (acc / 0.99).powi(6);
//             #[cfg(feature="debug_perf_rating")]
//             println!("diff:{diff}, acc: {acc} = perf {perf}");
//             perf
//         }))
// }}


// pub async fn calc_diff(
//     map: &BeatmapMeta, 
//     mode_override: String
// ) -> TatakuResult<Box<dyn DiffCalc>> {{
//     let playmode = map.check_mode_override(mode_override);

//     get_gamemode_info(&playmode)
//         .ok_or_else(|| TatakuError::GameMode(GameModeError::UnknownGameMode))?
//         .create_diffcalc(map).await
// }}

