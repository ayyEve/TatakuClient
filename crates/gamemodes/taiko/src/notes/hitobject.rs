use crate::prelude::*;
use engine::gameplay::HitObject;

pub trait TaikoHitObject: HitObject + Send + Sync {
    fn is_kat(&self) -> bool { false } // needed for diff calc and autoplay

    #[cfg(feature="graphics")] fn get_sv(&self) -> f32;
    #[cfg(feature="graphics")] fn set_sv(&mut self, sv: f32);

    /// does this hit object play a finisher sound when hit?
    #[cfg(feature="gameplay")] fn finisher_sound(&self) -> bool { false }

    /// used by autoplay, is this note a finisher?
    fn is_finisher(&self) -> bool { false }

    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool;

    /// returns true if a finisher was successfully hit
    fn check_finisher(
        &self,
        _hit_type: HitType,
        _time: f32,
        _game_speed: f32
    ) -> bool { false }

    #[cfg(feature="graphics")]
    fn get_playfield(&self) -> Arc<Playfield>;
    fn set_settings(&mut self, settings: Arc<Settings>);

    #[cfg(feature="graphics")]
    fn x_at(&self, time: f32) -> f32 {
        // (self.time() - time) * self.get_sv()
        ((self.time() - time) / SV_OVERRIDE)
            * self.get_sv()
            * self.get_playfield().size.x
    }
    #[cfg(feature="graphics")]
    fn end_x_at(&self, time: f32) -> f32 {
        ((self.end_time(0.0) - time) / SV_OVERRIDE)
            * self.get_sv()
            * self.get_playfield().size.x
    }

    #[cfg(feature="graphics")]
    fn time_at(&self, x: f32) -> f32 {
        -(x / self.get_sv()) + self.time()
    }

    fn hit_type(&self) -> HitType {
        if self.is_kat() { HitType::Kat } else { HitType::Don }
    }

    fn was_hit(&self) -> bool;
    fn force_hit(&mut self) {}

    fn hit(&mut self, _time: f32, _hit_type: HitType) -> bool { false }
    fn miss(&mut self, _time: f32) {}

    fn hits_to_complete(&self) -> u32 { 1 }

    #[cfg(feature="graphics")]
    fn playfield_changed(&mut self, _new_playfield: Arc<Playfield>);

    /// only used by spinners
    fn set_required_hits(&mut self, _required_hits: u16) {}

    /// used if no_finisher mod is enabled/disabled
    fn toggle_finishers(&mut self, _enabled: bool) {}
}
