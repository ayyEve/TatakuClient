use crate::prelude::*;


pub struct ManiaDifficultyCalculator {}

#[async_trait]
impl DiffCalc<super::super::Game> for ManiaDifficultyCalculator {
    async fn new(_g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    async fn calc(&mut self, _mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}