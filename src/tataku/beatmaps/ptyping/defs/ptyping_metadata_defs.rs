use crate::prelude::*;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingMetadataText {
    pub unicode: Option<String>,
    pub ascii: Option<String>
}
impl PTypingMetadataText {
    pub fn get_string(&self) -> String {
        self.ascii.clone().or_else(||self.unicode.clone()).unwrap_or_default()
    }
}


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingBeatmapInfo {
    pub description: String,
    pub difficulty_name: PTypingMetadataText,
    pub mapper: PTypingMapperDef,
    pub preview_time: f32
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingMapperDef {
    pub user_id: u64,
    pub username: String,
    pub online: bool
}


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingBeatmapMetadataDef {
    pub backing_languages: Vec<u8>,
    pub tags: Vec<String>
}



#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingFileCollectionDef {
    pub audio: PTypingFileDef,
    pub background: Option<PTypingFileDef>,
    pub background_video: Option<PTypingFileDef>,
}


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingFileDef {
    /// NOT ACTUALLY PATH, its just the display text for this file. use the hash for the path (no ext)
    pub path: String,
    pub hash: String
}