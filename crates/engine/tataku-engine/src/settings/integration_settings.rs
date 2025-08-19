use crate::prelude::*;

#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug, Default2, PartialEq)]
#[serde(default)]
pub struct IntegrationSettings {
    #[default(true)]
    #[setting(text="Discord Integration")]
    pub discord: bool,

    #[setting(text="LastFM Integration")]
    pub lastfm: bool,

    #[default(true)]
    #[setting(text="OS Media Controls")]
    pub media_controls: bool,

    #[subsetting()]
    pub osu: OsuIntegration,
}

#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct OsuIntegration {
    #[category(text="Osu Integration")]
    #[setting(text="Osu Username")]
    pub username: String,
    #[setting(text="Osu Password", password=true)]
    pub password: String,
    #[setting(text="Osu Api Key", password=true)]
    pub api_key: String,
}
