use crate::*;
use engine::gameplay::info::GamemodeInfo;

pub trait BeatmapAnimation: Send + Sync {
    fn update(&mut self, time: f32);
    fn draw(&self, list: &mut graphics::RenderableCollection);

    fn window_size_changed(&mut self, _size: tataku::Vector2) {}
    fn fit_to_area(&mut self, _playfield: engine::gameplay::PlayfieldNonsense) {}

    fn reset(&mut self);

    fn use_gamemode_playfield(&self, gamemode: &GamemodeInfo) -> bool;
}

#[derive(Default, Copy, Clone)]
pub struct EmptyAnimation;
impl BeatmapAnimation for EmptyAnimation {
    fn update(&mut self, _: f32) {}
    fn draw(&self, _: &mut graphics::RenderableCollection) {}
    fn reset(&mut self) {}

    fn use_gamemode_playfield(&self, _gamemode: &GamemodeInfo) -> bool { false }
}
