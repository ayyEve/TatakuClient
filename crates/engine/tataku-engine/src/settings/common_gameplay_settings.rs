use crate::prelude::*;

#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Default2, PartialEq, Debug)]
#[serde(default)]
pub struct CommonGameplaySettings {
    #[default(Key::Equals)]
    #[setting(text="Increase Offset")]
    pub key_offset_up: Key,

    #[default(Key::Minus)]
    #[setting(text="Decrease Offset")]
    pub key_offset_down: Key,

    #[default(Key::Grave)]
    #[setting(text="Restart Map Key")]
    pub map_restart_key: Key,

    #[default(200.0)]
    #[setting(text="Restart Map Hold Time", range(0.0, 1000.0))]
    pub map_restart_delay: f32,

    #[setting(text="Allow Settings Menu In-Game")]
    pub allow_ingame_settings: bool,

    // duration bar settings

    /// color of duration to go (bg)
    #[default(Color::from_hex("#66666680"))]
    pub duration_color: Color,

    /// color of duration completed
    #[default(Color::from_hex("#666F"))]
    pub duration_color_full: Color,
    
    /// color of duration border
    #[default(Color::from_hex("#000"))]
    pub duration_border_color: Color,


    // health bar
    /// colors for healthbar at %s
    /// ie [0%-full], [0%-50%, 50%-full], or [0%-33%, 33%-66%, 66%-full], etc
    #[default(vec![Color::from_hex("#0F0")])]
    pub healthbar_colors: Vec<Color>,

    /// color of healthbar background
    #[default(Color::from_hex("#66666680"))]
    pub healthbar_bg_color: Color,

    /// color of healthbar border
    #[default(Color::from_hex("#000"))]
    pub healthbar_border_color: Color,



    // hit indicators
    /// how long should a hit indicator be drawn for?
    #[default(500.0)]
    #[setting(text="Hit Indicator Draw Time", range(100.0, 1000.0))]
    pub hit_indicator_draw_duration: f32,

    /// how long should a hit indicator be drawn for?
    #[setting(text="Use Draw Time for Animations")]
    pub use_indicator_draw_duration_for_animations: bool,
}
