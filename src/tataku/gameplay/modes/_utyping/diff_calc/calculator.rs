use crate::prelude::*;

pub struct UTypingDifficultyCalculator {}

#[async_trait]
impl DiffCalc<super::super::UTypingGame> for UTypingDifficultyCalculator {
    async fn new(_g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    async fn calc(&mut self, _mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}