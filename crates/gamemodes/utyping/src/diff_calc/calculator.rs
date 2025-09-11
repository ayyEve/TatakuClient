use crate::prelude::*;

use engine::{
    game::diffcalc::*,
    beatmaps::{
        BeatmapMeta,
    },
    gameplay::{
        mods::ModManager,
    }
};

pub struct UTypingDifficultyCalculator {}
impl DiffCalc for UTypingDifficultyCalculator {
    fn new(_g: &BeatmapMeta, _: &engine::Settings) -> tataku::Result<Self> {
        Ok(Self {})
    }

    fn calc(&mut self, _mods: &ModManager) -> tataku::Result<DiffCalcSummary> {
        Ok(DiffCalcSummary::default())
    }
}