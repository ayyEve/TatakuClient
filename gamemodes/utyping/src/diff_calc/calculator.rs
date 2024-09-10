use crate::prelude::*;

pub struct UTypingDifficultyCalculator {}

#[async_trait]
impl DiffCalc for UTypingDifficultyCalculator {
    async fn new(_g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    async fn calc(&mut self, _mods: &ModManager) -> TatakuResult<DiffCalcSummary> {
        Ok(Default::default())
    }
}