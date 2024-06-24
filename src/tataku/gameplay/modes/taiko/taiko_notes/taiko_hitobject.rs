use crate::prelude::*;
use super::super::prelude::*;

pub trait TaikoHitObject: HitObject + Send + Sync {
    fn is_kat(&self) -> bool { false } // needed for diff calc and autoplay

    fn get_sv(&self) -> f32;
    fn set_sv(&mut self, sv:f32);
    /// does this hit object play a finisher sound when hit?
    fn finisher_sound(&self) -> bool { false }

    /// used by autoplay, is this note a finisher?
    fn is_finisher(&self) -> bool { false }

    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool;

    /// returns true if a finisher was successfully hit
    fn check_finisher(&self, _hit_type:HitType, _time:f32, _game_speed: f32) -> bool { false }

    fn get_playfield(&self) -> Arc<TaikoPlayfield>;
    fn set_settings(&mut self, settings: Arc<TaikoSettings>);


    fn x_at(&self, time:f32) -> f32 {
        // (self.time() - time) * self.get_sv()
        ((self.time() - time) / SV_OVERRIDE) * self.get_sv() * self.get_playfield().size.x
    }
    fn end_x_at(&self, time:f32) -> f32 {
        ((self.end_time(0.0) - time) / SV_OVERRIDE) * self.get_sv() * self.get_playfield().size.x
    }

    fn time_at(&self, x: f32) -> f32 {
        -(x / self.get_sv()) + self.time()
    }

    fn hit_type(&self) -> HitType {
        if self.is_kat() { HitType::Kat } else { HitType::Don }
    }
    
    fn was_hit(&self) -> bool;
    fn force_hit(&mut self) {}

    fn hit(&mut self, _time: f32) -> bool { false }
    fn miss(&mut self, _time: f32) {}

    fn hits_to_complete(&self) -> u32 { 1 }

    fn playfield_changed(&mut self, _new_playfield: Arc<TaikoPlayfield>);

    /// only used by spinners
    fn set_required_hits(&mut self, _required_hits:u16) {}

    /// used if no_finisher mod is enabled/disabled
    fn toggle_finishers(&mut self, _enabled: bool) {}
}



#[derive(Clone)]
pub struct HitCircleImageHelper {
    circle: Image,
    overlay: Image,
}
impl HitCircleImageHelper {
    pub async fn new(settings: &Arc<TaikoSettings>, hit_type: HitType, finisher: bool, skin_manager: &mut SkinManager) -> Option<Self> {
        let color = match hit_type {
            HitType::Don => settings.don_color.color,
            HitType::Kat => settings.kat_color.color,
        };

        let (radius, hitcircle) = if finisher {
            (settings.note_radius * settings.big_note_multiplier, "taikobigcircle")
        } else {
            (settings.note_radius, "taikohitcircle")
        };

        let mut circle = skin_manager.get_texture(hitcircle, true).await;
        if let Some(circle) = &mut circle {
            let scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;

            circle.pos = Vector2::ZERO;
            circle.scale = scale;
            circle.color = color;
        }

        let mut overlay = skin_manager.get_texture(hitcircle.to_owned() + "overlay", true).await;
        if let Some(overlay) = &mut overlay {
            let scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;

            overlay.pos = Vector2::ZERO;
            overlay.scale = scale;
            overlay.color = color;
        }

        if overlay.is_none() || circle.is_none() { return None }

        Some(Self {
            circle: circle.unwrap(),
            overlay: overlay.unwrap(),
        })
    }

    pub fn set_pos(&mut self, pos: Vector2) {
        self.circle.pos  = pos; // truncate(pos);
        self.overlay.pos = pos; // truncate(pos);
    }
    pub fn draw(&mut self, list: &mut RenderableCollection) {
        list.push(self.circle.clone());
        list.push(self.overlay.clone());
    }

    pub fn update_settings(&mut self, settings: Arc<TaikoSettings>, finisher: bool) {
        let radius = if finisher {
            settings.note_radius * settings.big_note_multiplier
        } else {
            settings.note_radius
        };

        let scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
        self.circle.scale = scale;
        self.overlay.scale = scale;
    }
}




// fn truncate(v: Vector2) -> Vector2 {
//     Vector2::new(
//         v.x.trunc(),
//         v.y.trunc()
//     )
// }