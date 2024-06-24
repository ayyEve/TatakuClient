use crate::prelude::*;

// hitobject trait, implemented by anything that should be hit
#[async_trait]
pub trait HitObject: Send + Sync {
    fn note_type(&self) -> NoteType;

    /// time in ms of this hit object
    fn time(&self) -> f32;
    /// when should the hitobject be considered "finished", should the miss hitwindow be applied (specifically for notes)
    fn end_time(&self, hitwindow_miss:f32) -> f32;

    async fn update(&mut self, time: f32);
    async fn draw(&mut self, time: f32, list: &mut RenderableCollection);

    /// set this object back to defaults
    async fn reset(&mut self);

    async fn time_jump(&mut self, _new_time: f32) {}

    async fn reload_skin(&mut self, _skin_manager: &mut SkinManager) {}

    
    fn beat_happened(&mut self, _pulse_length: f32) {}
    fn kiai_changed(&mut self, _is_kiai: bool) {}
}
