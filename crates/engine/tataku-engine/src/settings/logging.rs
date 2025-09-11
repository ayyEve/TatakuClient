use crate::*;
use common::reflect::*;
use tataku_client_proc_macros::Settings;


/// helper so i dont need to recompile the game every time i want to change what things are logged
#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct LoggingSettings {
    #[setting(text="Extra Online Logging")]
    pub extra_online_logging: bool,
    pub render_update_logging: bool,
}
