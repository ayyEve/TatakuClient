use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BackgroundGameSettings {
    /// whether to have gameplay in the main menu bg or not
    #[serde(alias="enabled")]
    pub main_menu_enabled: bool,

    /// whether to have gameplay in the beatmap select menu bg or not
    pub beatmap_select_enabled: bool,

    /// gameplay alpha multiplier
    pub opacity: f32,
    /// hitsound volume multiplier
    pub hitsound_volume: f32,
    /// what mode should be playing?
    pub mode: PlayMode,
}
impl Default for BackgroundGameSettings {
    fn default() -> Self {
        Self { 
            main_menu_enabled: true,
            beatmap_select_enabled: true,
            opacity: 0.5,
            hitsound_volume: 0.3,
            mode: "osu".to_owned()
        }
    }
}