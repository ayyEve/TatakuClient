use crate::prelude::*;


pub struct ManiaDifficultyCalculator {}

impl DiffCalc<super::super::Game> for ManiaDifficultyCalculator {
    fn new(g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}