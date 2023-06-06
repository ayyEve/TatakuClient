use crate::prelude::*;

const APPROACH_CIRCLE_MULT:f32 = 4.0;

pub struct ApproachCircle {
    image: Option<Image>,
    base_pos: Vector2,
    pos: Vector2,
    radius: f32,
    scaling_helper: Arc<ScalingHelper>,
    depth: f32,
    alpha: f32,
    color: Color,

    preempt: f32,
    time: f32,
    time_diff: f32,
}
impl ApproachCircle {
    pub fn new(base_pos:Vector2, time: f32, radius:f32, preempt:f32, depth:f32, color: Color, scaling_helper: Arc<ScalingHelper>) -> Self {
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
    pub fn scale_changed(&mut self, new_scale: Arc<ScalingHelper>, new_radius: f32) {
        self.scaling_helper = new_scale;
        self.pos = self.scaling_helper.scale_coords(self.base_pos);
        self.radius = new_radius;
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
        let lerp_amount = self.time_diff / self.preempt;
        let scale = f32::lerp(1.0, APPROACH_CIRCLE_MULT, lerp_amount); // TODO: allow other lerps?

        if let Some(mut tex) = self.image.clone() {
            tex.depth = self.depth - 100.0;
            tex.pos = self.pos;
            tex.color = self.color.alpha(self.alpha);
            tex.scale = Vector2::ONE * self.scaling_helper.scaled_cs * scale;

            list.push(tex)
        } else {
            list.push(Circle::new(
                Color::TRANSPARENT_WHITE,
                self.depth - 100.0,
                self.pos,
                self.radius * scale, // self.radius is already accounting for the scaled_cs
                Some(Border::new(self.color.alpha(self.alpha), OSU_NOTE_BORDER_SIZE * self.scaling_helper.scaled_cs))
            ))
        }
    }
}
