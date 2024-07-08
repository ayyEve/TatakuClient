use crate::prelude::*;

const APPROACH_CIRCLE_MULT:f32 = 4.0;

/// for some reason in tataku, approach circles at scale 1 are too big, so we fudge the scale with this to make it better (not perfect, idk why its broken in the first place)
const APPROACH_CIRCLE_SCALE:f32 = 0.90;

pub struct ApproachCircle {
    image: Option<Image>,
    base_pos: Vector2,
    pos: Vector2,
    radius: f32,
    scaling_helper: Arc<ScalingHelper>,
    alpha: f32,
    color: Color,

    preempt: f32,
    time: f32,
    time_diff: f32,

    pub easing_type: Easing
}
impl ApproachCircle {
    pub fn new(base_pos:Vector2, time: f32, radius:f32, preempt:f32, scaling_helper: Arc<ScalingHelper>) -> Self {
        Self {
            base_pos,
            pos: scaling_helper.scale_coords(base_pos),
            time,
            radius,
            preempt,
            color: Color::WHITE,
            scaling_helper,

            alpha: 0.0,
            image: None,
            time_diff: time,
            easing_type: Easing::Linear
        }
    }
    pub fn scale_changed(&mut self, new_scale: Arc<ScalingHelper>, new_radius: f32) {
        self.scaling_helper = new_scale;
        self.pos = self.scaling_helper.scale_coords(self.base_pos);
        self.radius = new_radius;
    }
    pub async fn reload_texture(&mut self, source: &TextureSource, skin_manager: &mut SkinManager) {
        self.image = skin_manager.get_texture("approachcircle", source, SkinUsage::Gamemode, false).await;
    }

    pub fn update(&mut self, map_time: f32) {
        self.time_diff = self.time - map_time;
    }

    pub fn reset(&mut self) {
        self.time_diff = 9999.0;
        self.alpha = 0.0;
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn draw(&self, list: &mut RenderableCollection) {
        let lerp_amount = self.time_diff / self.preempt;
        let scale = self.easing_type.run_easing(1.0, APPROACH_CIRCLE_MULT, lerp_amount.max(0.0));

        if let Some(mut tex) = self.image.clone() {
            tex.pos = self.pos;
            tex.color = self.color.alpha(self.alpha);
            tex.scale = Vector2::ONE * self.scaling_helper.scaled_cs * scale * APPROACH_CIRCLE_SCALE;

            list.push(tex)
        } else {
            list.push(Circle::new(
                self.pos,
                self.radius * scale, // self.radius is already accounting for the scaled_cs
                Color::TRANSPARENT_WHITE,
                Some(Border::new(self.color.alpha(self.alpha), OSU_NOTE_BORDER_SIZE * self.scaling_helper.scaled_cs))
            ))
        }
    }
}
