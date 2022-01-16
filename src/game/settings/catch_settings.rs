use piston::Key;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CatchSettings {
    // keys
    pub left_key: Key,
    pub right_key: Key,
    pub dash_key: Key,
}
impl Default for CatchSettings {
    fn default() -> Self {
        Self {
            // keys
            left_key: Key::Left,
            right_key: Key::Right,
            dash_key: Key::LShift,
        }
    }
}