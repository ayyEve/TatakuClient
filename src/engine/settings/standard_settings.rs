use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[derive(Settings)]
#[Setting(prefix="standard_settings")]
pub struct StandardSettings {
    // input
    #[Setting(text="Osu Key 1")]
    pub left_key: Key,
    #[Setting(text="Osu Key 2")]
    pub right_key: Key,
    #[Setting(text="Ignore Mouse Buttons")]
    pub ignore_mouse_buttons: bool,

    // playfield
    pub playfield_x_offset: f64,
    pub playfield_y_offset: f64,
    pub playfield_scale: f64,
    pub playfield_snap: f64,
    pub playfield_movelines_thickness: f64,

    // display
    #[Setting(text="Follow Points")]
    pub draw_follow_points: bool,
    pub combo_colors: Vec<String>,
    #[Setting(text="Show x300s")]
    pub show_300s: bool,

    // special effects
    #[Setting(text="Hit Ripples")]
    pub hit_ripples: bool,
    #[Setting(text="Slider Tick Ripples")]
    pub slider_tick_ripples: bool,
    #[Setting(text="Ripple HitCircles")]
    pub ripple_hitcircles: bool,
    #[Setting(text="Ripple Scale", min=0.1, max=5.0)]
    pub ripple_scale: f64,
    #[Setting(text="Slider Tick Ripples Above")]
    pub slider_tick_ripples_above: bool,
    #[Setting(text="Combo Color Approach Circles")]
    pub approach_combo_color: bool,

    #[Setting(text="Beatmap Combo Colors")]
    pub use_beatmap_combo_colors: bool,
}
impl StandardSettings {
    pub fn get_playfield(&self) -> (f64, Vector2) {
        (self.playfield_scale, Vector2::new(self.playfield_x_offset, self.playfield_y_offset))
    }
}
impl Default for StandardSettings {
    fn default() -> Self {
        Self {
            // keys
            left_key: Key::S,
            right_key: Key::D,
            ignore_mouse_buttons: false,

            playfield_x_offset: 0.0,
            playfield_y_offset: 0.0,
            playfield_scale: 0.8,
            playfield_snap: 20.0,
            playfield_movelines_thickness: 2.0,

            draw_follow_points: true,
            show_300s: true,

            combo_colors: vec![
                "#FFC000".to_owned(),
                "#00CA00".to_owned(),
                "#127CFF".to_owned(),
                "#F21839".to_owned()
            ],

            hit_ripples: true,
            ripple_hitcircles: false,
            ripple_scale: 2.0,
            slider_tick_ripples: true,
            slider_tick_ripples_above: true,
            approach_combo_color: true,

            use_beatmap_combo_colors: true,
        }
    }
}
