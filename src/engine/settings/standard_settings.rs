use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[derive(Settings)]
pub struct StandardSettings {
    // input
    #[Setting(text="Osu Key 1")]
    pub left_key: Key,
    #[Setting(text="Osu Key 2")]
    pub right_key: Key,
    #[Setting(text="Ignore Mouse Buttons")]
    pub ignore_mouse_buttons: bool,

    #[Setting(text="Allow manual input with Relax")]
    pub manual_input_with_relax: bool,
    

    // playfield
    pub playfield_x_offset: f32,
    pub playfield_y_offset: f32,
    pub playfield_scale: f32,
    pub playfield_snap: f32,
    pub playfield_movelines_thickness: f32,

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
    pub ripple_scale: f32,
    #[Setting(text="Slider Tick Ripples Above")]
    pub slider_tick_ripples_above: bool,
    #[Setting(text="Combo Color Approach Circles")]
    pub approach_combo_color: bool,

    #[Setting(text="Beatmap Combo Colors")]
    pub use_beatmap_combo_colors: bool,

    #[Setting(text="Use Skin Judgments")]
    pub use_skin_judgments: bool,


    /// min is 0.00001 because @ 0.0 it shows the shoddy slider rendering (try it and see!)
    #[Setting(text="Slider Body Alpha", min=0.00001, max=1.0)]
    pub slider_body_alpha: f32,
    #[Setting(text="Slider Border Alpha", min=0.0, max=1.0)]
    pub slider_border_alpha: f32,

    #[Setting(text="Playfield Alpha", min=0.0, max=1.0)]
    pub playfield_alpha: f32,
}
impl StandardSettings {
    pub fn get_playfield(&self) -> (f32, Vector2) {
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
            manual_input_with_relax: false,

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
            use_skin_judgments: true,
            slider_body_alpha: 0.8,
            slider_border_alpha: 1.0,
            playfield_alpha: 0.5
        }
    }
}
