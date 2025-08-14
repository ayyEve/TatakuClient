use crate::prelude::*;
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct BeatmapLink {
    #[serde(alias="@hash")]
    pub beatmap_hash: String,

    #[serde(alias="@title")]
    pub beatmap_title: String,

    #[serde(alias="@link", default)]
    pub download_link: Option<String>,
}
impl FromStr for BeatmapLink {
    type Err = TatakuError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        quick_xml::de::from_str(s)
            .map_err(TatakuError::from_err)
    }
}
impl std::fmt::Display for BeatmapLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = &self.beatmap_title;
        let hash = &self.beatmap_hash;
        if let Some(link) = &self.download_link {
            write!(f, "<beatmapLink hash=\\{hash}\" title=\"{title}\" link=\"{link}\" />")
        } else {
            write!(f, "<beatmapLink hash=\\{hash}\" title=\"{title}\" />")
        }
    }
}
