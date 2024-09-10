use crate::prelude::*;

pub trait ManiaHitObject: HitObject {
    fn hit(&mut self, time:f32);
    fn release(&mut self, _time:f32) {}
    fn miss(&mut self, time:f32);
    fn was_hit(&self) -> bool { false }
    fn get_hitsound(&self) -> &Vec<Hitsound>;

    fn set_sv_mult(&mut self, sv: f32);
    fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>);
    fn playfield_changed(&mut self, playfield: Arc<ManiaPlayfield>);
    fn set_skin_settings(&mut self, settings: Option<Arc<ManiaSkinSettings>>);
}