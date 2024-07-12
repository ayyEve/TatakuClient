use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Clone, Debug, Serialize, PartialEq, SettingsDeserialize)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[serde(default)]
pub struct OsuSettings {
    // input
    #[cfg_attr(feature="graphics", Setting(text="Osu Key 1"))]
    pub left_key: Key,
    #[cfg_attr(feature="graphics", Setting(text="Osu Key 2"))]
    pub right_key: Key,
    #[cfg_attr(feature="graphics", Setting(text="Osu Smoke Key"))]
    pub smoke_key: Key,

    #[cfg_attr(feature="graphics", Setting(text="Ignore Mouse Buttons"))]
    pub ignore_mouse_buttons: bool,

    #[cfg_attr(feature="graphics", Setting(text="Allow manual input with Relax"))]
    pub manual_input_with_relax: bool,
    

    // playfield
    pub playfield_x_offset: f32,
    pub playfield_y_offset: f32,
    pub playfield_scale: f32,
    pub playfield_snap: f32,
    pub playfield_movelines_thickness: f32,

    // display
    #[cfg_attr(feature="graphics", Setting(text="Follow Points"))]
    pub draw_follow_points: bool,
    pub combo_colors: Vec<String>,
    #[cfg_attr(feature="graphics", Setting(text="Show x300s"))]
    pub show_300s: bool,

    // special effects
    #[cfg_attr(feature="graphics", Setting(text="Hit Ripples"))]
    pub hit_ripples: bool,
    #[cfg_attr(feature="graphics", Setting(text="Slider Tick Ripples"))]
    pub slider_tick_ripples: bool,
    #[cfg_attr(feature="graphics", Setting(text="Ripple HitCircles"))]
    pub ripple_hitcircles: bool,
    #[cfg_attr(feature="graphics", Setting(text="Ripple Scale", min=0.1, max=5.0))]
    pub ripple_scale: f32,
    #[cfg_attr(feature="graphics", Setting(text="Slider Tick Ripples Above"))]
    pub slider_tick_ripples_above: bool,
    #[cfg_attr(feature="graphics", Setting(text="Combo Color Approach Circles"))]
    pub approach_combo_color: bool,

    #[cfg_attr(feature="graphics", Setting(text="Beatmap Combo Colors"))]
    pub use_beatmap_combo_colors: bool,

    #[cfg_attr(feature="graphics", Setting(text="Use Skin Judgments"))]
    pub use_skin_judgments: bool,

    #[cfg_attr(feature="graphics", Setting(text="Use beatmap skin"))]
    pub beatmap_skin: bool,

    /// min is 0.00001 because @ 0.0 it shows the shoddy slider rendering (try it and see!)
    #[cfg_attr(feature="graphics", Setting(text="Slider Body Alpha", min=0.00001, max=1.0))]
    pub slider_body_alpha: f32,
    #[cfg_attr(feature="graphics", Setting(text="Slider Border Alpha", min=0.0, max=1.0))]
    pub slider_border_alpha: f32,
    #[cfg_attr(feature="graphics", Setting(text="Use Skin Slider Body Color"))]
    pub use_skin_slider_body_color: bool,

    #[cfg_attr(feature="graphics", Setting(text="Playfield Alpha", min=0.0, max=1.0))]
    pub playfield_alpha: f32,

    #[cfg_attr(feature="graphics", Setting(text="Slider Render Targets"))]
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
