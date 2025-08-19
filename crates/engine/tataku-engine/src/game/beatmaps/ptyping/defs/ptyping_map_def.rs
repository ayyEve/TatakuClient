use crate::prelude::*;

#[derive(Clone, Default, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingMapDef {
    pub id: String,
    pub beatmaps: Vec<PTypingBeatmapDef>,
    
    pub source: String,
    pub artist: PTypingMetadataText,
    pub title: PTypingMetadataText,
}
