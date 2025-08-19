// pub mod diff_calc;
mod stats;
mod game_mode;
mod gameplay_mods;
mod gameplay_widgets;
mod gameplay_manager;
mod gameplay_helpers;
#[cfg(feature="dynamic_gamemodes")]
mod gamemode_library;

pub use stats::*;
pub use game_mode::*;
pub use gameplay_mods::*;
pub use gameplay_widgets::*;
pub use gameplay_manager::*;
pub use gameplay_helpers::*;

#[cfg(feature="dynamic_gamemodes")]
pub use gamemode_library::*;

use crate::prelude::*;

pub trait DiffCalc: Send + Sync {
    fn new(g: &BeatmapMeta, settings: &Settings) -> TatakuResult<Self> where Self:Sized;
    fn calc(&mut self, mods: &ModManager) -> TatakuResult<DiffCalcSummary>;
}

#[derive(Default)]
#[derive(Serialize)]
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

#[derive(Reflect)]
#[derive(Clone, Debug, Default)]
pub struct GamemodeInfos {
    #[reflect(skip)]
    pub by_id: Arc<HashMap<&'static str, GamemodeInfo>>,
    pub by_num: Arc<Vec<GamemodeInfo>>,

    #[cfg(feature="dynamic_gamemodes")]
    _libraries: Arc<Vec<libloading::Library>>,
}
impl GamemodeInfos {
    #[cfg(not(feature="dynamic_gamemodes"))]
    pub fn new(list: Vec<GamemodeInfo>) -> Self {
        Self {
            by_id: Arc::new(list.iter()
                .map(|i| (i.id, *i))
                .collect()
            ),
            by_num: Arc::new(list),
        }
    }

    #[cfg(feature="dynamic_gamemodes")]
    pub fn new(list: Vec<GamemodeLibrary>) -> Self {

        let (libraries, by_num): (_, Vec<GamemodeInfo>) = list.into_iter()
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
    pub fn get_info(&self, gamemode: &str) -> TatakuResult<&GamemodeInfo> {
        self.by_id
            .get(gamemode)
            .ok_or(TatakuError::GameMode(GameModeError::UnknownGameMode))
    }

    pub fn get_playmode_actual<'a>(
        &self, 
        playmode: &'a str, 
        beatmap: Option<&'a BeatmapMeta>
    ) -> &'a str {
        let Ok(info) = self.get_info(playmode) 
        else { return playmode };
        
        beatmap
            .filter(|b| !info.can_load_beatmap(&b.beatmap_type))
            .map_or(playmode, |b| &*b.mode)
    }
}
