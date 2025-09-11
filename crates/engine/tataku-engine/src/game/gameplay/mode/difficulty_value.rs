use crate::*;
use common::reflect::*;

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default2)]
pub struct DifficultyValue {
    /// internal id of this value
    #[default("none")]
    pub id: &'static str,
    
    /// display name for this value
    #[default("None")]
    pub name: &'static str,

    /// can custom values be set for this?
    pub modifiable: bool,

    /// is this a whole number, or a floating point number?
    #[default(DifficultyNumberType::WholeNumber)]
    pub number_type: DifficultyNumberType,

    /// the minimum value this can be
    pub min: f32,

    /// the maximum value this can be
    pub max: f32,

    /// how much to step by (if modifiable)
    pub step: Option<f32>,

    /// what unit to append to the diff string
    pub unit: Option<&'static str>,

    /// What special display callback should be used?
    #[reflect(skip)]
    pub display: Option<fn(f32) -> String>,

    /// get the value for this from the map and mods provided
    #[reflect(skip)]
    #[default(|_| 0.0)]
    pub get_diff_value: fn(&GetDiffValue) -> f32,
}
impl DifficultyValue {
    pub fn format(&self, num: f32) -> String {
        let num = if let Some(display) = self.display {
            display(num)
        } else {
            match self.number_type {
                DifficultyNumberType::Float => tataku::format_float(num, 2),
                DifficultyNumberType::WholeNumber => tataku::format_number(num as u64),
            }
        };
        format!("{}: {num}{}", self.name, self.unit.unwrap_or_default())
    }
}

impl AsRef<str> for DifficultyValue {
    fn as_ref(&self) -> &str {
        self.id
    }
}
impl PartialEq for DifficultyValue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for DifficultyValue {}
impl std::hash::Hash for DifficultyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Reflect)]
#[derive(Copy, Clone, Debug)]
pub enum DifficultyNumberType {
    WholeNumber,
    Float,
}




pub const DIFFICULTY_DIFF_VALUE: DifficultyValue = DifficultyValue {
    id: "diff",
    name: "Difficulty",
    modifiable: false,
    number_type: DifficultyNumberType::Float,
    min: 0.0,
    max: 200.0,
    step: None,
    unit: Some("*"),
    display: None,
    get_diff_value: |info| info.diff,
};

pub const BPM_DIFF_VALUE: DifficultyValue = DifficultyValue {
    id: "bpm",
    name: "BPM",
    modifiable: false,
    number_type: DifficultyNumberType::Float,
    min: 0.0,
    max: 999999.0,
    step: None,
    unit: Some("bpm"),
    display: None,
    get_diff_value: |info| info.map.bpm_min * info.mods.get_speed(),
};

pub const DURATION_DIFF_VALUE: DifficultyValue = DifficultyValue {
    id: "duration",
    name: "Duration",
    modifiable: false,
    number_type: DifficultyNumberType::Float,
    min: 0.0,
    max: 999999.0,
    step: None,
    unit: None,
    display: Some(display_time),
    get_diff_value: |info| info.map.duration * info.mods.speed.as_f32(),
};

fn display_time(ms: f32) -> String {
    let seconds_total = ms / 1000.0;
    let mins = (seconds_total / 60.0).floor();
    let secs = seconds_total % 60.0;
    format!("{mins:.0}:{secs:.0}")
}

pub struct GetDiffValue<'a> {
    pub map: &'a BeatmapMeta,
    pub mods: &'a gameplay::mods::ModManager,
    pub diff: f32,
}
