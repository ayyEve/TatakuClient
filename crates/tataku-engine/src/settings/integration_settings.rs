use crate::prelude::*;

#[derive(Clone, Serialize, PartialEq, Debug)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct IntegrationSettings {
    #[cfg_attr(feature="graphics", setting(text="Discord Integration"))]
    pub discord: bool,
    #[cfg_attr(feature="graphics", setting(text="LastFM Integration"))]
    pub lastfm: bool,
    #[cfg_attr(feature="graphics", setting(text="OS Media Controls"))]
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
