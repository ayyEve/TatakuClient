use crate::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct DifficultyValue {
    /// internal id of this value
    pub id: &'static str,
    
    /// display name for this value
    pub name: &'static str,

    /// can custom values be set for this?
    pub modifiable: bool,

    /// is this a whole number, or a floating point number?
    pub number_type: DifficultyNumberType,

    /// the minimum value this can be
    pub min: f32,

    /// the maximum value this can be
    pub max: f32,

    /// how much to step by (if modifiable)
    pub step: Option<f32>,

    /// what unit to append to the diff string
    pub unit: Option<&'static str>,

    /// get the value for this from the map and mods provided
    pub get_diff_value: fn(&BeatmapMetaWithDiff, &ModManager) -> f32,
}
impl DifficultyValue {
    pub const DEFAULT: Self = Self {
        id: "none",
        name: "None",
        modifiable: false,
        number_type: DifficultyNumberType::WholeNumber,
        min: 0.0,
        max: 0.0,
        step: None,
        unit: None,
        get_diff_value: |_,_| 0.0,
    };

    pub fn format(&self, num: f32) -> String {
        let num = match self.number_type {
            DifficultyNumberType::Float => crate::format_float(num, 2),
            DifficultyNumberType::WholeNumber => crate::format_number(num as u64),
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
        self.id.hash(state)
    }
}

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
    get_diff_value: |map, _| map.diff.unwrap_or_default(),
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
    get_diff_value: |map, _|map.bpm_min,
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
    get_diff_value: |map, mods| map.secs(mods.speed.as_f32()),
};
