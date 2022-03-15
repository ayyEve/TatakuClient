use crate::prelude::*;


pub struct ManiaDifficultyCalculator {}

impl DiffCalc<super::super::Game> for ManiaDifficultyCalculator {
    fn new(_g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, _mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}