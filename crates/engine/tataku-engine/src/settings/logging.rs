use crate::*;
use common::reflect::*;
use tataku_client_proc_macros::Settings;

#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct LoggingSettings {
    #[setting(text="Extra Online Logging")]
    pub extra_online_logging: bool,
    pub render_update_logging: bool,
}
