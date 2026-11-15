use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Easing,
    Vector2,
};

use engine::graphics;

const APPROACH_CIRCLE_MULT:f32 = 4.0;

/// for some reason in tataku, approach circles at scale 1 are too big, so we fudge the scale with this to make it better (not perfect, idk why its broken in the first place)
const APPROACH_CIRCLE_SCALE:f32 = 0.90;

#[derive(Default)]
pub struct ApproachCircle {
    image: Option<graphics::Image>,
    pub easing_type: Easing
}
impl ApproachCircle {
    pub fn new() -> Self {
        Self {
            image: None,
            easing_type: Easing::Linear
        }
    }

    #[cfg(feature="graphics")]
    pub fn reload_texture(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        self.image = skin_manager.get_texture(
            Path::new("approachcircle"),
            source,
            graphics::SkinUsage::Gamemode,
            false
        );
    }

    #[cfg(feature="graphics")]
    pub fn draw(
        &self,
        note_time: f32,
        color_index: usize,
        shell: &mut DrawShell
    ) {
        let color = shell.combo_color(color_index)
            .with_alpha(shell.alpha);

        let lerp_amount = (note_time - shell.time) / shell.fade_in_time;

        let scale = self.easing_type.run_easing(
            1.0,
            APPROACH_CIRCLE_MULT,
            lerp_amount.clamp(0.0, 1.0),
        );

        if let Some(mut tex) = self.image.clone() {
            tex.color = color;

            let transform = graphics::Transform {
                origin: tex.size() / 2.0,
                scale: Vector2::ONE * CIRCLE_RADIUS_BASE * scale * APPROACH_CIRCLE_SCALE,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(tex.with_transform(
                shell.transform * transform.matrix()
            ));
        } else {
            let transform = graphics::Transform {
                scale: Vector2::ONE * CIRCLE_RADIUS_BASE * scale,
                ..graphics::Transform::IDENTITY
            };

            shell.list.push(graphics::Circle::new(
                Color::TRANSPARENT,
            ).border(Border::new(
                color,
                OSU_NOTE_BORDER_SIZE * shell.coords.cs)
            ).with_transform(
                shell.transform * transform.matrix()
            ));
        }
    }
}
