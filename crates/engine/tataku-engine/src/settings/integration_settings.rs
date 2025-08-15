use crate::prelude::*;

#[derive(Clone, Serialize, PartialEq, Debug)]
#[derive(Reflect, Settings, SettingsDeserialize)]
#[serde(default)]
pub struct IntegrationSettings {
    #[setting(text="Discord Integration")]
    pub discord: bool,
    #[setting(text="LastFM Integration")]
    pub lastfm: bool,
    #[setting(text="OS Media Controls")]
    pub media_controls: bool,

    #[subsetting()]
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
#[derive(Reflect, Settings, SettingsDeserialize)]
#[serde(default)]
pub struct OsuIntegration {
    #[setting(text="Osu Integration")]
    #[setting(text="Osu Username")]
    pub username: String,
    #[setting(text="Osu Password", password=true)]
    pub password: String,
    #[setting(text="Osu Api Key", password=true)]
    pub api_key: String,
}