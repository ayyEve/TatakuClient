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

    #[cfg_attr(feature="graphics", subsetting())]
    pub osu: OsuIntegration,
}
impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            discord: true,
            lastfm: false,
            media_controls: true,
            osu: OsuIntegration::default(),
        }
    }
}

#[derive(Clone, Serialize, PartialEq, Debug, Default)]
#[cfg_attr(feature="graphics", derive(Settings))]
#[derive(Reflect, SettingsDeserialize)]
#[serde(default)]
pub struct OsuIntegration {
    #[cfg_attr(feature="graphics", setting(text="Osu Integration"))]
    #[cfg_attr(feature="graphics", setting(text="Osu Username"))]
    pub username: String,
    #[cfg_attr(feature="graphics", setting(text="Osu Password", password=true))]
    pub password: String,
    #[cfg_attr(feature="graphics", setting(text="Osu Api Key", password=true))]
    pub api_key: String,
}