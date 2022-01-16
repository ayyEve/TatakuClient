use piston::Key;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TaikoSettings {
    // sv
    pub static_sv: bool,
    pub sv_multiplier: f32,

    // keys
    pub left_kat: Key,
    pub left_don: Key,
    pub right_don: Key,
    pub right_kat: Key,

    pub ignore_mouse_buttons: bool
}
impl Default for TaikoSettings {
    fn default() -> Self {
        Self {
            // keys
            left_kat: Key::D,
            left_don: Key::F,
            right_don: Key::J,
            right_kat: Key::K,

            // sv
            static_sv: false,
            sv_multiplier: 1.0,
            
            ignore_mouse_buttons: false
        }
    }
}
