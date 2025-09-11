use crate::*;
use common::reflect::*;

#[cfg(feature="graphics")]
pub trait GamemodeSettings: Reflect + settings::MakeSettingsMenu + std::fmt::Debug {
    fn to_value(&self) -> serde_json::Value;
    fn duplicate_settings(&self) -> Box<dyn GamemodeSettings>;
}

#[cfg(not(feature="graphics"))]
pub trait GamemodeSettings: Reflect + std::fmt::Debug {
    fn to_value(&self) -> serde_json::Value;
    fn duplicate_settings(&self) -> Box<dyn GamemodeSettings>;
}

common::impl_downcast!(GamemodeSettings);

#[repr(C)]
#[derive(Reflect)]
#[derive(Copy, Clone, Debug2)]
pub struct GamemodeInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub about: &'static str,
    pub author: &'static str,
    pub author_contact: &'static str,
    pub bug_report_url: &'static str,

    pub mods: &'static [gameplay::mods::GameplayModGroupStatic],
    pub stat_groups: &'static [gameplay::stats::StatGroup],
    pub judgments: &'static [gameplay::judgments::HitJudgment],
    pub diff_values: &'static [gameplay::difficulty_value::DifficultyValue],
    pub available_widgets: &'static [gameplay::widgets::GameplayWidgetBuilder],

    #[debug(skip)]
    #[reflect(skip)]
    pub calc_acc: fn(&common::Score) -> f32,

    #[debug(skip)]
    #[reflect(skip)]
    pub calc_perf: fn(CalcPerfInfo) -> f32,

    #[debug(skip)]
    #[reflect(skip)]
    pub can_load_beatmap: fn(&beatmaps::BeatmapType) -> bool,
    
    #[debug(skip)]
    #[reflect(skip)]
    pub stats_from_groups: fn(&HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<gameplay::stats::StatsInfo>,

    #[debug(skip)]
    #[reflect(skip)]
    pub create_game: fn(&beatmaps::Beatmap, &Settings) -> tataku::TatakuResult<Box<dyn gameplay::GameMode>>,

    #[debug(skip)]
    #[reflect(skip)]
    pub create_diffcalc: fn(&BeatmapMeta, &Settings) -> tataku::TatakuResult<Box<dyn game::diffcalc::DiffCalc>>,


    #[debug(skip)]
    #[reflect(skip)]
    pub deserialize_settings: fn(serde_json::Value) -> Option<Box<dyn GamemodeSettings>>,
    
    #[debug(skip)]
    #[reflect(skip)]
    pub serialize_settings: fn(Box<dyn GamemodeSettings>) -> serde_json::Value,
}
impl GamemodeInfo {
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
        available_widgets: &[],
        calc_acc: |_| 0.0,
        calc_perf: Self::default_calc_perf,
        stats_from_groups: |_| Vec::new(),
        can_load_beatmap: |_| false,
        create_game: |_, _| Err(errors::game_mode::GameModeError::UnknownGameMode.into()),
        create_diffcalc: |_,_| Err(errors::game_mode::GameModeError::UnknownGameMode.into()),
        deserialize_settings: |_| None, 
        serialize_settings: |_| panic!("serialize_settings not implemented!")
    };


    // TODO:
    fn default_calc_perf(data: CalcPerfInfo) -> f32 {
        data.map_difficulty * (data.accuracy / 0.99).powi(6)
    }

    pub fn calc_acc(&self, score: &common::Score) -> f32 {
        (self.calc_acc)(score)
    }
    pub fn calc_perf(&self, data: CalcPerfInfo<'_>) -> f32 {
        (self.calc_perf)(data)
    }

    pub fn stats_from_groups(
        &self, 
        stats: &HashMap<String, HashMap<String, Vec<f32>>>
    ) -> Vec<gameplay::stats::StatsInfo> {
        (self.stats_from_groups)(stats)
    }

    pub fn can_load_beatmap(&self, map: &beatmaps::BeatmapType) -> bool {
        (self.can_load_beatmap)(map)
    }


    pub fn create_game(
        &self, 
        map: &beatmaps::Beatmap, 
        settings: &Settings
    ) -> tataku::TatakuResult<Box<dyn gameplay::GameMode>> {
        (self.create_game)(map, settings)
    }
    
    pub fn create_diffcalc(
        &self, 
        map: &BeatmapMeta, 
        settings: &Settings
    ) -> tataku::TatakuResult<Box<dyn engine::game::diffcalc::DiffCalc>> {
        (self.create_diffcalc)(map, settings)
    }


    pub fn deserialize_settings(
        &self, 
        value: serde_json::Value
    ) -> Option<Box<dyn GamemodeSettings>> {
        (self.deserialize_settings)(value)
    }
    pub fn serialize_settings(
        &self, 
        s: Box<dyn GamemodeSettings>
    ) -> serde_json::Value {
        (self.serialize_settings)(s)
    }
}
impl Default for GamemodeInfo {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl AsRef<str> for GamemodeInfo {
    fn as_ref(&self) -> &str {
        self.id
    }
}

pub struct CalcPerfInfo<'a> {
    pub score: &'a common::Score,
    pub accuracy: f32,
    pub map_difficulty: f32,
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
    pub fn get_info(&self, gamemode: &str) -> tataku::TatakuResult<&GamemodeInfo> {
        Ok(self.by_id
            .get(gamemode)
            .ok_or(errors::game_mode::GameModeError::UnknownGameMode)?)
    }

    pub fn get_playmode_actual<'a>(
        &self, 
        playmode: &'a str, 
        beatmap: Option<&'a engine::BeatmapMeta>
    ) -> &'a str {
        let Ok(info) = self.get_info(playmode) 
        else { return playmode };
        
        beatmap
            .filter(|b| !info.can_load_beatmap(&b.beatmap_type))
            .map_or(playmode, |b| &*b.mode)
    }
}
