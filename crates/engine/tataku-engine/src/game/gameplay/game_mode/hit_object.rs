use crate::prelude::*;
use tataku_graphics::prelude::*;

// hitobject trait, implemented by anything that should be hit
pub trait HitObject: Send + Sync {
    fn note_type(&self) -> NoteType;

    /// time in ms of this hit object
    fn time(&self) -> f32;
    /// when should the hitobject be considered "finished", should the miss hitwindow be applied (specifically for notes)
    fn end_time(&self, hitwindow_miss: f32) -> f32;

    fn update(&mut self, time: f32);

    #[cfg(feature="graphics")]
    fn draw(&mut self, time: f32, list: &mut RenderableCollection);

    /// set this object back to defaults
    fn reset(&mut self);

    fn time_jump(&mut self, _new_time: f32) {}

    #[cfg(feature="graphics")]
    fn reload_skin(&mut self, _source: &TextureSource, _skin_manager: &mut dyn SkinProvider) {}

    
    fn beat_happened(&mut self, _pulse_length: f32) {}
    fn kiai_changed(&mut self, _is_kiai: bool) {}
}
