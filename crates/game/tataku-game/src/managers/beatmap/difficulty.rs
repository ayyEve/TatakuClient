use crate::prelude::*;
use common::reflect::*;

#[derive(Reflect)]
#[reflect(dont_clone)]
#[derive(Debug, Default)]
#[reflect(display="display")]
pub(crate) struct BeatmapDifficulty {
    pub diff: f32,
    pub info: Box<str>,
}
impl std::fmt::Display for BeatmapDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.info.fmt(f)
    }
}
