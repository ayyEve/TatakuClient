use crate::prelude::*;

#[derive(Clone, Serialize, PartialEq, Debug)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct CursorSettings {
    #[cfg_attr(feature="graphics", setting(text="Cursor Color"))]
    pub cursor_color: SettingsColor,
    #[cfg_attr(feature="graphics", setting(text="Cursor Scale", min=0.1, max=10.0))]
    pub cursor_scale: f32,
    #[cfg_attr(feature="graphics", setting(text="Cursor Border", min=0.1, max=5.0))]
    pub cursor_border: f32,
    #[cfg_attr(feature="graphics", setting(text="Cursor Border Color"))]
    pub cursor_border_color: SettingsColor,

    #[cfg_attr(feature="graphics", setting(text="Cursor Ripples"))]
    pub cursor_ripples: bool,
    #[cfg_attr(feature="graphics", setting(text="Cursor Ripple Color"))]
    pub cursor_ripple_color: SettingsColor,
    #[cfg_attr(feature="graphics", setting(text="Cursor Ripple Radius"))]
    pub cursor_ripple_final_radius: f32,

    #[cfg_attr(feature="graphics", setting(text="Use Beatmap Cursor"))]
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
