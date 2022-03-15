use crate::prelude::*;

pub struct UTypingDifficultyCalculator {}

impl DiffCalc<super::super::UTypingGame> for UTypingDifficultyCalculator {
    fn new(_g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, _mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}