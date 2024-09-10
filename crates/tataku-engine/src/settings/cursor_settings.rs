use crate::prelude::*;
#[derive(Clone, Serialize, PartialEq, Debug)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct CursorSettings {
    #[cfg_attr(feature="graphics", Setting(text="Cursor Color"))]
    pub cursor_color: SettingsColor,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Scale", min=0.1, max=10.0))]
    pub cursor_scale: f32,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Border", min=0.1, max=5.0))]
    pub cursor_border: f32,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Border Color"))]
    pub cursor_border_color: SettingsColor,

    #[cfg_attr(feature="graphics", Setting(text="Cursor Ripples"))]
    pub cursor_ripples: bool,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Ripple Color"))]
    pub cursor_ripple_color: SettingsColor,
    #[cfg_attr(feature="graphics", Setting(text="Cursor Ripple Scale"))]
    pub cursor_ripple_final_scale: f32,

    #[cfg_attr(feature="graphics", Setting(text="Use Beatmap Cursor"))]
    pub beatmap_cursor: bool,
}
impl Default for CursorSettings {
    fn default() -> Self {
        Self {
            cursor_scale: 1.0,
            cursor_border: 1.5,
            cursor_color: Color::from_hex("#ffff32".to_owned()).into(),
            cursor_border_color: Color::from_hex("#000".to_owned()).into(),
            cursor_ripples: true,
            cursor_ripple_color: Color::from_hex("#000".to_owned()).into(),
            cursor_ripple_final_scale: 1.5,
            beatmap_cursor: true,
        }
    }
}