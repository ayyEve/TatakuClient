use crate::prelude::*;

#[async_trait]
pub trait CustomCursor {
    async fn update(&mut self, time: f32);
    #[cfg(feature="graphics")]
    async fn reload_skin(&mut self, skin_manager: &mut dyn SkinProvider);

    async fn draw_above(&mut self, list: &mut RenderableCollection);
    async fn draw_below(&mut self, _list: &mut RenderableCollection) {}

    fn left_pressed(&mut self, pressed: bool);
    fn right_pressed(&mut self, pressed: bool);
    fn cursor_pos(&mut self, pos: Vector2);
    async fn render_trail(&mut self, time: f32);
}