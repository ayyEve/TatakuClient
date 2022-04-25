use crate::prelude::*;

/// helper so i dont need to recompile the game every time i want to change what things are logged
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoggingSettings {
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