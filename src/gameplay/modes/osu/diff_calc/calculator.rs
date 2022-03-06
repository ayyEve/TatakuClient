use crate::prelude::*;


pub struct OsuDifficultyCalculator {}

impl DiffCalc<super::super::Game> for OsuDifficultyCalculator {
    fn new(g: &BeatmapMeta) -> TatakuResult<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32> {
        Ok(0.0)
    }
}