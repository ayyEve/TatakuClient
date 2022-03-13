use crate::prelude::*;

pub struct UTypingDifficultyCalculator {}

impl DiffCalc<super::super::UTypingGame> for UTypingDifficultyCalculator {
    fn new(g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}