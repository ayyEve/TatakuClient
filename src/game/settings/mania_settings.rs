use crate::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ManiaSettings {
    // sv
    pub static_sv: bool,
    pub sv_multiplier: f32,

    // playfield settings
    pub playfield_settings: Vec<ManiaPlayfieldSettings>,

    /// col_count [col_num, 0 based]
    /// ie for 4k, key 2: mania_keys\[3]\[1]
    pub keys: Vec<Vec<Key>>,

    /// how much to change the sv by when a sv change key is pressed
    pub sv_change_delta: f32,

    pub judgements_per_column: bool,
    
    /// how far from the hit position should hit indicators be?
    pub judgement_indicator_offset: f64,

}
impl Default for ManiaSettings {
    fn default() -> Self {
        Self {
            // keys
            keys: vec![
                vec![Key::Space], // 1k
                vec![Key::F, Key::J], // 2k
                vec![Key::F, Key::Space, Key::J], // 3k
                vec![Key::D, Key::F, Key::J, Key::K], // 4k
                vec![Key::D, Key::F, Key::Space, Key::J, Key::K], // 5k
                vec![Key::S, Key::D, Key::F, Key::J, Key::K, Key::L], // 6k
                vec![Key::S, Key::D, Key::F, Key::Space, Key::J, Key::K, Key::L], // 7k
                vec![Key::A, Key::S, Key::D, Key::F, Key::J, Key::K, Key::L, Key::Semicolon], // 8k
                vec![Key::A, Key::S, Key::D, Key::F, Key::Space, Key::J, Key::K, Key::L, Key::Semicolon], // 9k
            ],

            // playfield settings
            playfield_settings: vec![
                ManiaPlayfieldSettings::new("1 Key"),
                ManiaPlayfieldSettings::new("2 Key"),
                ManiaPlayfieldSettings::new("3 Key"),
                ManiaPlayfieldSettings::new("4 Key"),
                ManiaPlayfieldSettings::new("5 Key"),
                ManiaPlayfieldSettings::new("6 Key"),
                ManiaPlayfieldSettings::new("7 Key"),
                ManiaPlayfieldSettings::new("8 Key"),
                ManiaPlayfieldSettings::new("9 Key"),
            ],

            // sv
            static_sv: false,
            sv_multiplier: 1.0,
            sv_change_delta: 0.3,


            // other
            judgements_per_column: false,
            judgement_indicator_offset: 200.0
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ManiaPlayfieldSettings {
    /// name of this config
    pub name: String,

    /// y pos of the hit area
    /// 
    /// if not upside-down, y is window_height - this
    pub hit_pos: f64,

    /// how wide is a note column?
    pub column_width: f64,

    /// how wide is the gap between columns?
    pub column_spacing: f64,

    /// how tall is a note?
    pub note_height: f64,

    /// how offset is the playfield?
    pub x_offset: f64,

    /// how thicc is the note border?
    pub note_border_width: f64,

    /// do the notes scroll up?
    pub upside_down: bool,
}
impl ManiaPlayfieldSettings {
    pub fn new(name: &str) -> Self{
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub fn hit_y(&self) -> f64 {
        if self.upside_down {
            self.hit_pos
        } else {
            Settings::window_size().y - self.hit_pos
        }
    }

    #[inline(always)]
    pub fn note_size(&self) -> Vector2 {
        Vector2::new(
            self.column_width,
            self.note_height
        )
    }
}
impl Default for ManiaPlayfieldSettings {
    fn default() -> Self {
        Self {
            name: "unknown".to_owned(),

            hit_pos: 200.0,
            column_width: 100.0,
            column_spacing: 5.0,
            note_height: 50.0,
            x_offset: 0.0,

            note_border_width: 1.4,

            upside_down: false,
        }
    }
}
