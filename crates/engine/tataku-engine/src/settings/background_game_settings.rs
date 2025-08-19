use crate::prelude::*;

#[derive(Reflect, Settings)]
#[derive(Serialize, DeserializeSettings)]
#[derive(Clone, Debug, Default2, PartialEq)]
#[serde(default)]
pub struct BackgroundGameSettings {
    /// gameplay alpha multiplier
    #[default(0.5)] pub opacity: f32,
    
    /// hitsound volume multiplier
    #[default(0.3)] pub hitsound_volume: f32,

    /// what mode should be playing?   
    #[default("osu".to_owned())] pub mode: String,
}
