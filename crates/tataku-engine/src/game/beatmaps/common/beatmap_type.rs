use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[derive(Reflect)]
pub enum BeatmapType {
    Unknown,
    Adofai,
    Osu,
    Quaver,
    Stepmania,
    Tja,
    UTyping
}
impl Default for BeatmapType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Into<u8> for BeatmapType {
    fn into(self) -> u8 {
        match self {
            BeatmapType::Unknown => 0,
            BeatmapType::Adofai => 1,
            BeatmapType::Osu => 2,
            BeatmapType::Quaver => 3,
            BeatmapType::Stepmania => 4,
            BeatmapType::Tja => 5,
            BeatmapType::UTyping => 6,
        }
    }
}
impl From<u8> for BeatmapType {
    fn from(n: u8) -> Self {
        match n {
            0 => BeatmapType::Unknown,
            1 => BeatmapType::Adofai,
            2 => BeatmapType::Osu,
            3 => BeatmapType::Quaver,
            4 => BeatmapType::Stepmania,
            5 => BeatmapType::Tja,
            6 => BeatmapType::UTyping,
            _ => BeatmapType::Unknown,
        }
    }
}

impl Into<MapGame> for BeatmapType {
    fn into(self) -> MapGame {
        match self {
            BeatmapType::Osu => MapGame::Osu,
            BeatmapType::Quaver => MapGame::Quaver,
            other => MapGame::Other(format!("{other:?}").to_lowercase())
        }
    }
}