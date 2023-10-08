use crate::prelude::*;

#[derive(Clone, Serialize, PartialEq)]
#[derive(Settings, SettingsDeserialize)]
#[serde(default)]
pub struct IntegrationSettings {
    #[Setting(text="Discord Integration")]
    pub discord: bool,
    #[Setting(text="LastFM Integration")]
    pub lastfm: bool,
    #[Setting(text="OS Media Controls")]
    pub media_controls: bool,
}
impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            discord: true,
            lastfm: false,
            media_controls: true,
        }
    }
}
