use crate::prelude::*;

pub struct UTypingDifficultyCalculator {}

#[async_trait]
impl DiffCalc for UTypingDifficultyCalculator {
    fn new(_g: &BeatmapMeta, _: &Settings) -> TatakuResult<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, _mods: &ModManager) -> TatakuResult<DiffCalcSummary> {
        Ok(Default::default())
    }
}