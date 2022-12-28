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



#[derive(Clone)]
pub struct HitCircleImageHelper {
    pub pos: Vector2,
    pub circle: Image,
    pub overlay: Image,
}
impl HitCircleImageHelper {
    pub async fn new(pos: Vector2, scaling_helper: &Arc<ScalingHelper>, depth: f64, color: Color) -> Option<Self> {
        let mut circle = SkinManager::get_texture("hitcircle", true).await;
        if let Some(circle) = &mut circle {
            circle.depth = depth;
            circle.pos = pos;
            circle.scale = Vector2::ONE * scaling_helper.scaled_cs;
            circle.color = color;
        }

        let mut overlay = SkinManager::get_texture("hitcircleoverlay", true).await;
        if let Some(overlay) = &mut overlay {
            overlay.depth = depth - 0.0000001;
            overlay.pos = pos;
            overlay.scale = Vector2::ONE * scaling_helper.scaled_cs;
        }

        if overlay.is_none() || circle.is_none() {return None}

        Some(Self {
            circle: circle.unwrap(),
            overlay: overlay.unwrap(),
            pos: scaling_helper.descale_coords(pos)
        })
    }

    
    pub fn playfield_changed(&mut self, new_scale: &Arc<ScalingHelper>) {
        self.overlay.pos = new_scale.scale_coords(self.pos);
        self.overlay.scale = Vector2::ONE * new_scale.scaled_cs;

        self.circle.pos   = self.overlay.pos;
        self.circle.scale = self.overlay.scale;
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.circle.color.a = alpha;
        self.overlay.color.a = alpha;
    }

    pub fn draw(&mut self, list: &mut RenderableCollection) {
        list.push(self.circle.clone());
        list.push(self.overlay.clone());
    }
}


pub struct ApproachCircle {
    image: Option<Image>,
    base_pos: Vector2,
    pos: Vector2,
    radius: f64,
    scaling_helper: Arc<ScalingHelper>,
    depth: f64,
    alpha: f32,
    color: Color,

    preempt: f32,
    time: f32,
    time_diff: f32,
}
impl ApproachCircle {
    pub fn new(base_pos:Vector2, time: f32, radius:f64, preempt:f32, depth:f64, color: Color, scaling_helper: Arc<ScalingHelper>) -> Self {
        Self {
            base_pos,
            pos: scaling_helper.scale_coords(base_pos),
            time,
            radius,
            preempt,
            depth, 
            color,
            scaling_helper,

            alpha: 0.0,
            image: None,
            time_diff: time
        }
    }
    pub fn scale_changed(&mut self, new_scale: Arc<ScalingHelper>) {
        self.scaling_helper = new_scale;
        self.pos = self.scaling_helper.scale_coords(self.base_pos);
    }
    pub async fn reload_texture(&mut self) {
        self.image = SkinManager::get_texture("approachcircle", true).await;
    }

    pub fn update(&mut self, map_time: f32, alpha: f32) {
        self.alpha = alpha;
        self.time_diff = self.time - map_time;
    }

    pub fn reset(&mut self) {
        self.time_diff = 9999.0;
        self.alpha = 0.0;
    }

    pub fn draw(&self, list: &mut RenderableCollection) {
        if let Some(mut tex) = self.image.clone() {
            tex.depth = self.depth - 100.0;
            // i think this is incorrect
            let scale = 1.0 + (self.time_diff as f64 / self.preempt as f64) * (APPROACH_CIRCLE_MULT - 1.0);

            tex.pos = self.pos;
            tex.color = self.color.alpha(self.alpha);
            tex.scale = Vector2::ONE * scale * self.scaling_helper.scaled_cs;

            list.push(tex)
        } else {
            list.push(Circle::new(
                Color::TRANSPARENT_WHITE,
                self.depth - 100.0,
                self.pos,
                self.radius + (self.time_diff as f64 / self.preempt as f64) * (APPROACH_CIRCLE_MULT * CIRCLE_RADIUS_BASE * self.scaling_helper.scaled_cs),
                Some(Border::new(self.color.alpha(self.alpha), NOTE_BORDER_SIZE * self.scaling_helper.scaled_cs))
            ))
        }
    }
}
