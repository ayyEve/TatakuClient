use crate::prelude::*;
use tataku_client_proc_macros::Settings;

/// helper so i dont need to recompile the game every time i want to change what things are logged
#[derive(Copy, Clone, Serialize, Debug, Default, PartialEq)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct LoggingSettings {
    #[cfg_attr(feature="graphics", Setting(text="Extra Online Logging"))]
    pub extra_online_logging: bool,
    pub render_update_logging: bool,
}
