use crate::prelude::*;

#[async_trait]
pub trait BeatmapAnimation: Send + Sync {
    async fn update(&mut self, time: f32, manager: &GameplayManager);
    async fn draw(&self, list: &mut RenderableCollection);

    fn window_size_changed(&mut self, _size: Vector2) {}
    fn fit_to_area(&mut self, _bounds: Bounds) {}

    fn reset(&mut self);
}

#[derive(Default, Copy, Clone)]
pub struct EmptyAnimation;

#[async_trait]
impl BeatmapAnimation for EmptyAnimation {
    async fn update(&mut self, _: f32, _: &GameplayManager) {}
    async fn draw(&self, _: &mut RenderableCollection) {}
    fn reset(&mut self) {}
}