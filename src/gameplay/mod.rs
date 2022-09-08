pub mod modes;

// pub mod diff_calc;
mod game_mode;
mod ingame_manager;
mod beatmap_structs;
mod gameplay_helpers;

pub use game_mode::*;
pub use ingame_manager::*;
pub use beatmap_structs::*;
pub use gameplay_helpers::*;

use crate::prelude::*;
#[async_trait]
pub trait DiffCalc<G:GameMode> where Self:Sized {
    async fn new(g: &BeatmapMeta) -> TatakuResult<Self>;
    async fn calc(&mut self, mods: &ModManager) -> TatakuResult<f32>;
}