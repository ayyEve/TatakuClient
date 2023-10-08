use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Serialize, PartialEq)]
#[derive(Settings, SettingsDeserialize)]
#[serde(default)]
pub struct CommonGameplaySettings {
    #[Setting(text="Increase Offset")]
    pub key_offset_up: Key,
    #[Setting(text="Decrease Offset")]
    pub key_offset_down: Key,

    #[Setting(text="Restart Map Key")]
    pub map_restart_key: Key,
    #[Setting(text="Restart Map Hold Time", min=0, max=1000)]
    pub map_restart_delay: f32,

    #[Setting(text="Allow Settings Menu In-Game")]
    pub allow_ingame_settings: bool,

    // duration bar settings
    /// color of duration to go (bg)
    pub duration_color: Color,
    /// color of duration completed
    pub duration_color_full: Color,
    /// color of duration border
    pub duration_border_color: Color,


    // health bar
    /// colors for healthbar at %s
    /// ie [0%-full], [0%-50%, 50%-full], or [0%-33%, 33%-66%, 66%-full], etc
    pub healthbar_colors: Vec<Color>,
    /// color of healthbar background
    pub healthbar_bg_color: Color,
    /// color of healthbar border
    pub healthbar_border_color: Color,


    // hit indicators
    /// how long should a hit indicator be drawn for?
    #[Setting(text="Hit Indicator Draw Time", min=100, max=1000)]
    pub hit_indicator_draw_duration: f32,
    /// how long should a hit indicator be drawn for?
    #[Setting(text="Use Draw Time for Animations")]
    pub use_indicator_draw_duration_for_animations: bool,
}

impl Default for CommonGameplaySettings {
    fn default() -> Self {
        Self { 
            key_offset_up: Key::Equals,
            key_offset_down: Key::Minus,
            map_restart_key: Key::Grave,
            map_restart_delay: 200.0,
            allow_ingame_settings: true,

            // duration bar
            duration_color: Color::from_hex("#66666680"),
            duration_color_full: Color::from_hex("#666F"),
            duration_border_color: Color::from_hex("#000"),

            // healthbar
            healthbar_colors: vec![Color::from_hex("#0F0")],
            healthbar_bg_color: Color::from_hex("#66666680"),
            healthbar_border_color: Color::from_hex("#000"),

            // hit indicators
            hit_indicator_draw_duration: 500.0,
            use_indicator_draw_duration_for_animations: false,
        }
    }
}
