use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[derive(Reflect, Settings, SettingsDeserialize)]
#[serde(default)]
pub struct OsuSettings {
    // input
    #[setting(text="Osu Key 1")]
    pub left_key: Key,
    #[setting(text="Osu Key 2")]
    pub right_key: Key,
    #[setting(text="Osu Smoke Key")]
    pub smoke_key: Key,

    #[setting(text="Ignore Mouse Buttons")]
    pub ignore_mouse_buttons: bool,

    #[setting(text="Allow manual input with Relax")]
    pub manual_input_with_relax: bool,
    

    // playfield
    pub playfield_x_offset: f32,
    pub playfield_y_offset: f32,
    pub playfield_scale: f32,
    pub playfield_snap: f32,
    pub playfield_movelines_thickness: f32,

    // display
    #[setting(text="Follow Points")]
    pub draw_follow_points: bool,
    pub combo_colors: Vec<String>,
    #[setting(text="Show x300s")]
    pub show_300s: bool,

    // special effects
    #[setting(text="Hit Ripples")]
    pub hit_ripples: bool,
    #[setting(text="Slider Tick Ripples")]
    pub slider_tick_ripples: bool,
    #[setting(text="Ripple HitCircles")]
    pub ripple_hitcircles: bool,
    #[setting(text="Ripple Scale", range(0.1, 5.0))]
    pub ripple_scale: f32,
    #[setting(text="Slider Tick Ripples Above")]
    pub slider_tick_ripples_above: bool,
    #[setting(text="Combo Color Approach Circles")]
    pub approach_combo_color: bool,

    #[setting(text="Beatmap Combo Colors")]
    pub use_beatmap_combo_colors: bool,

    #[setting(text="Use Skin Judgments")]
    pub use_skin_judgments: bool,

    #[setting(text="Use beatmap skin")]
    pub beatmap_skin: bool,

    /// min is 0.00001 because @ 0.0 it shows the shoddy slider rendering (try it and see!)
    #[setting(text="Slider Body Alpha", range(0.00001, 1.0))]
    pub slider_body_alpha: f32,
    #[setting(text="Slider Border Alpha", range(0.0, 1.0))]
    pub slider_border_alpha: f32,
    #[setting(text="Use Skin Slider Body Color")]
    pub use_skin_slider_body_color: bool,

    #[setting(text="Playfield Alpha", range(0.0, 1.0))]
    pub playfield_alpha: f32,

    #[setting(text="Slider Render Targets")]
    pub slider_render_targets: bool,
}
impl OsuSettings {
    pub fn get_playfield(&self) -> (f32, Vector2) {
        (self.playfield_scale, Vector2::new(self.playfield_x_offset, self.playfield_y_offset))
    }
}
impl Default for OsuSettings {
    fn default() -> Self {
        Self {
            // keys
            left_key: Key::S,
            right_key: Key::D,
            smoke_key: Key::A,
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
            beatmap_skin: true,

            use_beatmap_combo_colors: true,
            use_skin_judgments: true,
            slider_body_alpha: 0.8,
            slider_border_alpha: 1.0,
            use_skin_slider_body_color: true,
            playfield_alpha: 0.5,

            slider_render_targets: false
        }
    }
}

impl GamemodeSettings for OsuSettings {
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn duplicate_settings(&self) -> Box<dyn GamemodeSettings> {
        Box::new(self.clone())
    }
}
