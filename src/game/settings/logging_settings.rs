use crate::prelude::*;

/// helper so i dont need to recompile the game every time i want to change what things are logged
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoggingSettings {
    pub extra_online_logging: bool,
}