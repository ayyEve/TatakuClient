use crate::*;
use tataku::Color;
use common::reflect::*;
use settings::SettingsColor;
use tataku_client_proc_macros::Settings;

#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug, Default2, PartialEq)]
#[serde(default)]
pub struct CursorSettings {
    #[default(Color::from_hex("#ffff32").into())]
    #[setting(text="Cursor Color")]
    pub cursor_color: SettingsColor,

    #[default(1.0)]
    #[setting(text="Cursor Scale", range(0.1, 10.0))]
    pub cursor_scale: f32,

    #[default(1.5)]
    #[setting(text="Cursor Border", range(0.1, 5.0))]
    pub cursor_border: f32,

    #[default(Color::from_hex("#000").into())]
    #[setting(text="Cursor Border Color")]
    pub cursor_border_color: SettingsColor,

    
    #[default(true)]
    #[setting(text="Cursor Ripples")]
    pub cursor_ripples: bool,

    #[default(Color::from_hex("#fff").into())]
    #[setting(text="Cursor Ripple Color")]
    pub cursor_ripple_color: SettingsColor,

    #[default(64.0)]
    #[setting(text="Cursor Ripple Radius")]
    pub cursor_ripple_final_radius: f32,

    #[default(true)]
    #[setting(text="Use Beatmap Cursor")]
    pub beatmap_cursor: bool,
}
