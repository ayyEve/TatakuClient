use crate::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardSettings {
    // input
    // #[setting(name="Left Key", input_type="KeyInput")]
    pub left_key: Key,
    pub right_key: Key,
    pub ignore_mouse_buttons: bool,

    // playfield
    pub playfield_x_offset: f64,
    pub playfield_y_offset: f64,
    pub playfield_scale: f64,
    pub playfield_snap: f64,
    pub playfield_movelines_thickness: f64,

    // display
    pub draw_follow_points: bool,
    pub combo_colors: Vec<String>,
    pub show_300s: bool,

    // special effects
    pub hit_ripples: bool,
    pub ripple_hitcircles: bool,
    pub ripple_scale: f64,
    pub slider_tick_ripples: bool,
    pub slider_tick_ripples_above: bool,
    pub approach_combo_color: bool,

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
