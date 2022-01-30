use crate::prelude::*;


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

    pub ignore_mouse_buttons: bool,
    pub controller_config: HashMap<String, TaikoControllerConfig>
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
            
            ignore_mouse_buttons: false,

            controller_config: HashMap::new()
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaikoControllerConfig {
    pub left_kat: ControllerInputConfig,
    pub left_don: ControllerInputConfig,
    pub right_don: ControllerInputConfig,
    pub right_kat: ControllerInputConfig,
}
impl TaikoControllerConfig {
    fn new_default(left_kat:u8, left_don:u8, right_don: u8, right_kat: u8) -> Self {
        Self {
            left_kat: ControllerInputConfig::new(Some(left_kat), None),
            left_don: ControllerInputConfig::new(Some(left_don), None),
            right_don: ControllerInputConfig::new(Some(right_don), None),
            right_kat: ControllerInputConfig::new(Some(right_kat), None)
        }
    }
    pub fn defaults(controller_name: Arc<String>) -> Self {
        match &**controller_name {
            "Taiko Controller" => Self::new_default(6, 10, 11, 7),
            "Xbox Controller" => Self::new_default(13, 12, 0, 1),
            "Wireless Controller" => Self::new_default(17, 15, 0, 2),

            _ => Self {
                left_kat: ControllerInputConfig::new(None, None),
                left_don: ControllerInputConfig::new(None, None),
                right_don:ControllerInputConfig::new(None, None),
                right_kat:ControllerInputConfig::new(None, None),
            }
        }
    }
}
