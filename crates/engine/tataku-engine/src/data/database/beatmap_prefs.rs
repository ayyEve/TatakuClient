use crate::*;
use common::reflect::*;

#[derive(Settings, Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Default2, Debug, PartialEq)]
#[serde(default)]
pub struct BeatmapPreferences {
    #[setting(text = "Audio Offset", range(-500.0, 500.0))]
    pub audio_offset: f32,
    
    #[default(true)]
    #[setting(text = "Storyboard")]
    pub storyboard: bool,

    #[default(true)]
    #[setting(text = "Beatmap Skin")]
    pub beatmap_skin: bool,

    // not yet implemented
    pub background_video: bool,
}


#[derive(Settings, Reflect)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug, Default2, PartialEq)]
#[serde(default)]
pub struct BeatmapPlaymodePreferences {
    #[default(1.0)]
    pub scroll_speed: f32,
}
