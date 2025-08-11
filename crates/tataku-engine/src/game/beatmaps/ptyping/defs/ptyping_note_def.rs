use crate::prelude::*;


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingNoteDef {
    pub time: f64,
    pub color: PTypingNoteColor,
    #[serde(with = "tataku_client_common::data::de_arc_str")]
    pub text: Arc<str>,
    pub settings: PTypingNoteSettings,
    pub typing_conversion: u8
}


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingNoteColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingNoteSettings {
    pub approach_modifier: Option<f32>
}