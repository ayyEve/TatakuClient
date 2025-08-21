use crate::prelude::*;
use tataku_client_proc_macros::Settings;

#[derive(Reflect, Settings)]
#[derive(Clone, PartialEq, Debug)]
#[derive(Serialize, DeserializeSettings)]
#[serde(default)]
pub struct TaikoSettings {
    // sv
    #[setting(text="SV Multiplier", range(1.0, 2.0))]
    pub sv_multiplier: f32,

    // size stuff
    #[setting(text="Note Radius", range(1.0, 100.0))]
    pub note_radius: f32,
    #[setting(text="Big Note Scale", range(1.0, 5.0))]
    pub big_note_multiplier: f32,

    // /// hit area, but calculated before use
    // #[serde(skip)]
    // pub hit_position: Vector2,
    pub hit_position_relative_to_window_size: bool,
    pub hit_position_relative_height_div: f32,
    #[setting(text="Playfield Horizontal Offset", range(0.0, 500.0))]
    pub playfield_x_offset: f32,
    #[setting(text="Playfield Vertical Offset", range(0.0, 200.0))]
    pub playfield_y_offset: f32,

    /// hit area raidus multiplier, 1.0 = note radius
    #[setting(text="Hit Area Radius Scale", range(1.0, 5.0))]
    pub hit_area_radius_mult: f32,
    /// playfield = note_radius * max(hit_area_radius_mult, big_note_mult) + this
    #[setting(text="Playfield Vertical Padding", range(0.0, 20.0))]
    pub playfield_height_padding: f32,

    #[setting(text="Use Skin Judgments")]
    pub use_skin_judgments: bool,
    
    /// how far above the hit position should hit indicators be?
    #[setting(text="Hit Judgment Y-Offset", range(0.0, 100.0))]
    pub judgement_indicator_offset: f32,
}
// impl TaikoSettings {
//     pub fn get_playfield(&self, width: f32, kiai: bool) -> Rectangle {
//         let height = self.note_radius * self.big_note_multiplier * 2.0 + self.playfield_height_padding;
//         Rectangle::new(
//             Vector2::new(0.0, self.hit_position.y - height / 2.0),
//             Vector2::new(width, height),
//             Color::new(0.1, 0.1, 0.1, 1.0),
//             if kiai {
//                 Some(Border::new(Color::YELLOW, 2.0))
//             } else {None}
//         )
//     }
// }
impl Default for TaikoSettings {
    fn default() -> Self {
        Self {

            // sv
            sv_multiplier: 1.1,
            
            // size stuff
            note_radius: 42.0,
            big_note_multiplier: 1.666666,
            hit_area_radius_mult: 1.2,
            playfield_height_padding: 8.0,
            // hit area stuff
            // hit_position: Vector2::ZERO,
            hit_position_relative_to_window_size: true,
            hit_position_relative_height_div: 1.375, // 3/8s the way down the screen
            playfield_x_offset: 200.0,
            playfield_y_offset: 0.0,
            
            judgement_indicator_offset: 0.0,
            use_skin_judgments: true
        }
    }
}

impl GamemodeSettings for TaikoSettings {
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
    fn duplicate_settings(&self) -> Box<dyn GamemodeSettings> {
        Box::new(self.clone())
    }
}
