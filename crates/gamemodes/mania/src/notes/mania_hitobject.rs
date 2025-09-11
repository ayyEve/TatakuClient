use crate::prelude::*;
use engine::*;

pub trait ManiaHitObject: gameplay::HitObject {
    fn hit(&mut self, time: f32);
    fn release(&mut self, _time: f32) {}
    fn was_hit(&self) -> bool { false }

    #[cfg(feature="gameplay")] fn get_hitsound(&self) -> &Vec<gameplay::Hitsound>;
    #[cfg(feature="graphics")] fn set_sv_mult(&mut self, sv: f32);
    #[cfg(feature="graphics")] fn set_position_function(&mut self, p: Arc<Vec<PositionPoint>>);
    #[cfg(feature="graphics")] fn playfield_changed(&mut self, playfield: Arc<ManiaPlayfield>);
}