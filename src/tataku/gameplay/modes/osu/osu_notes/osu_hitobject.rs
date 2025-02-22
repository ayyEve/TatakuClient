use crate::prelude::*;
use super::super::prelude::*;

#[async_trait]
pub trait OsuHitObject: HitObject {
    /// return the window-scaled coords of this object at time
    fn pos_at(&self, time:f32) -> Vector2;

    fn pending_combo(&mut self) -> Vec<(OsuHitJudgments, Vector2)> {Vec::new()}

    async fn playfield_changed(&mut self, new_scale: Arc<ScalingHelper>);
    fn set_settings(&mut self, settings: Arc<StandardSettings>);

    fn press(&mut self, _time:f32) {}
    fn release(&mut self, _time:f32) {}
    fn mouse_move(&mut self, pos:Vector2);

    fn get_preempt(&self) -> f32;
    fn point_draw_pos(&self, time: f32) -> Vector2;

    fn was_hit(&self) -> bool;

    fn get_hitsound(&self) -> Vec<Hitsound>;
    fn get_sound_queue(&mut self) -> Vec<Vec<Hitsound>> { vec![] }

    fn set_hitwindow_miss(&mut self, window: f32);


    fn miss(&mut self);
    fn hit(&mut self, time: f32);
    fn set_judgment(&mut self, _j:&OsuHitJudgments) {}
    fn set_ar(&mut self, ar: f32);

    fn check_distance(&self, mouse_pos: Vector2) -> bool;
    fn check_release_points(&mut self, _time: f32) -> OsuHitJudgments { OsuHitJudgments::Miss } // miss default, bc we only care about sliders

}
