use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
#[derive(Settings)]
#[Setting(prefix="integrations")]
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
