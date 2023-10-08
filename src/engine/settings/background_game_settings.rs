use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[derive(Settings, SettingsFormat)]
#[serde(default, from="BackgroundGameSettingsDeserializer")]
pub struct BackgroundGameSettings {
    /// whether to have gameplay in the main menu bg or not
    #[serde(alias="enabled")]
    #[Setting(text="Main Menu Background Gameplay")]
    pub main_menu_enabled: bool,

    /// whether to have gameplay in the beatmap select menu bg or not
    #[Setting(text="Map Select Background Gameplay")]
    pub beatmap_select_enabled: bool,

    /// whether to have gameplay in the settings menu bg or not
    #[Setting(text="Settings Background Gameplay")]
    pub settings_menu_enabled: bool,

    /// whether to have gameplay in the settings menu bg or not
    #[Setting(text="Multiplayer Background Gameplay")]
    pub multiplayer_menu_enabled: bool,

    /// gameplay alpha multiplier
    pub opacity: f32,
    /// hitsound volume multiplier
    pub hitsound_volume: f32,
    /// what mode should be playing?
    pub mode: String,
}
impl Default for BackgroundGameSettings {
    fn default() -> Self {
        Self { 
            main_menu_enabled: true,
            beatmap_select_enabled: true,
            settings_menu_enabled: true,
            multiplayer_menu_enabled: true,
            opacity: 0.5,
            hitsound_volume: 0.3,
            mode: "osu".to_owned()
        }
    }
}