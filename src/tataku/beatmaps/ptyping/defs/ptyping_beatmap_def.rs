use crate::prelude::*;


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingBeatmapDef {
    pub id: String,
    // breaks: Vec<>,
    pub hit_objects: Vec<PTypingNoteDef>,
    pub events: Vec<PTypingEventDef>,
    pub timing_points: Vec<PTypingTimingPointDef>,
    pub difficulty: PTypingDifficultyDef,
    pub info: PTypingBeatmapInfo,
    pub metadata: PTypingBeatmapMetadataDef,
    pub file_collection: PTypingFileCollectionDef,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingEventDef {
    pub start: f64,
    
    #[serde(deserialize_with = "infinity_reader")]
    pub end: f64,
    pub text: Option<String>,
    pub backing_type: u8
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingTimingPointDef {
    pub time: f64,
    pub tempo: f64,
    pub time_signature: f32
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PTypingDifficultyDef {
    pub strictness: f32,
}

/// helper for reading "Infinity" from json files when it should be an f64
fn infinity_reader<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    use std::fmt;
    use serde::de::{self, Visitor};

    struct InfinityReader;
    impl<'de> Visitor<'de> for InfinityReader {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("expected \"Infinty\" or f64")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            match value {
                "Infinity" => Ok(f64::INFINITY),
                _ => panic!("bad")
            }
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(v)
        }
        
        fn visit_f32<E: de::Error>(self, v: f32) -> Result<Self::Value, E> {
            Ok(v as f64)
        }
    }

    deserializer.deserialize_any(InfinityReader)
}
