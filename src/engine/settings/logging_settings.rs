use crate::prelude::*;
use tataku_client_proc_macros::Settings;

/// helper so i dont need to recompile the game every time i want to change what things are logged
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
#[derive(Settings)]
pub struct LoggingSettings {
    #[Setting(text="Extra Online Logging")]
    pub extra_online_logging: bool,
    pub render_update_logging: bool,
}

impl LoggingSettings {
    pub fn new() -> Self {
        Self {
            extra_online_logging: false,
            render_update_logging: false,
        }
    }
}