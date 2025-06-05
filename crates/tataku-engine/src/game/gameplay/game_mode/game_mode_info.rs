use crate::prelude::*;

#[cfg(feature="graphics")]
pub trait GamemodeSettings: Reflect + MakeSettingsMenu + std::fmt::Debug {
    fn to_value(&self) -> serde_json::Value;
}

#[cfg(not(feature="graphics"))]
pub trait GamemodeSettings: Reflect + std::fmt::Debug {
    fn to_value(&self) -> serde_json::Value;
}

impl_downcast!(GamemodeSettings);

#[repr(C)]
#[derive(Reflect, Debug2)]
#[derive(Copy, Clone)]
pub struct GamemodeInfo {
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
    pub available_widgets: &'static [GameplayWidgetBuilder],

    #[debug(skip)]
    #[reflect(skip)]
    pub calc_acc: fn(&Score) -> f32,

    #[debug(skip)]
    #[reflect(skip)]
    pub calc_perf: fn(CalcPerfInfo) -> f32,

    #[debug(skip)]
    #[reflect(skip)]
    pub can_load_beatmap: fn(&BeatmapType) -> bool,
    
    #[debug(skip)]
    #[reflect(skip)]
    pub stats_from_groups: fn(&HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<StatsInfo>,

    #[debug(skip)]
    #[reflect(skip)]
    pub create_game: fn(&Beatmap, &Settings) -> TatakuResult<Box<dyn GameMode>>,

    #[debug(skip)]
    #[reflect(skip)]
    pub create_diffcalc: fn(&BeatmapMeta, &Settings) -> TatakuResult<Box<dyn DiffCalc>>,


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
        create_game: |_, _| Err(GameModeError::UnknownGameMode.into()),
        create_diffcalc: |_,_| Err(GameModeError::UnknownGameMode.into()),
        deserialize_settings: |_| None, 
        serialize_settings: |_| panic!("serialize_settings not implemented!")
    };


    // TODO:
    fn default_calc_perf(data: CalcPerfInfo) -> f32 {
        data.map_difficulty * (data.accuracy / 0.99).powi(6)
    }

    pub fn calc_acc(&self, score: &Score) -> f32 {
        (self.calc_acc)(score)
    }
    pub fn calc_perf(&self, data: CalcPerfInfo<'_>) -> f32 {
        (self.calc_perf)(data)
    }

    pub fn stats_from_groups(&self, stats: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<StatsInfo> {
        (self.stats_from_groups)(stats)
    }

    pub fn can_load_beatmap(&self, map: &BeatmapType) -> bool {
        (self.can_load_beatmap)(map)
    }


    pub fn create_game(&self, map: &Beatmap, settings: &Settings) -> TatakuResult<Box<dyn GameMode>> {
        (self.create_game)(map, settings)
    }
    
    pub fn create_diffcalc(&self, map: &BeatmapMeta, settings: &Settings) -> TatakuResult<Box<dyn DiffCalc>> {
        (self.create_diffcalc)(map, settings)
    }


    pub fn deserialize_settings(&self, value: serde_json::Value) -> Option<Box<dyn GamemodeSettings>> {
        (self.deserialize_settings)(value)
    }
    pub fn serialize_settings(&self, s: Box<dyn GamemodeSettings>) -> serde_json::Value {
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
    pub score: &'a Score,
    pub accuracy: f32,
    pub map_difficulty: f32,
}
