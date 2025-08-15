use crate::prelude::*;

#[derive(Clone, Serialize, PartialEq, Debug)]
#[derive(Reflect, Settings, SettingsDeserialize)]
#[serde(default)]
pub struct CursorSettings {
    #[setting(text="Cursor Color")]
    pub cursor_color: SettingsColor,
    #[setting(text="Cursor Scale", range(0.1, 10.0))]
    pub cursor_scale: f32,
    #[setting(text="Cursor Border", range(0.1, 5.0))]
    pub cursor_border: f32,
    #[setting(text="Cursor Border Color")]
    pub cursor_border_color: SettingsColor,

    #[setting(text="Cursor Ripples")]
    pub cursor_ripples: bool,
    #[setting(text="Cursor Ripple Color")]
    pub cursor_ripple_color: SettingsColor,
    #[setting(text="Cursor Ripple Radius")]
    pub cursor_ripple_final_radius: f32,

    #[setting(text="Use Beatmap Cursor")]
    pub beatmap_cursor: bool,
}
impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            cursor_scale: 1.0,
            cursor_border: 1.5,
            cursor_color: Color::from_hex("#ffff32").into(),
            cursor_border_color: Color::from_hex("#000").into(),
            cursor_ripples: true,
            cursor_ripple_color: Color::from_hex("#fff").into(),
            cursor_ripple_final_radius: 64.0,
            beatmap_cursor: true,
        }
    }
}
