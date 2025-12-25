use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Vector2,
};

#[cfg(feature="graphics")]
use engine::graphics;

#[derive(Default, Clone)]
pub struct Note {
    pub time: f32,
    pub hit_time: f32,
    pub hit_type: HitType,
    pub base_finisher: bool,
    pub finisher: bool,
    pub hit: bool,
    pub missed: bool,

    #[cfg(feature="graphics")] pub speed: f32,

    #[cfg(feature="graphics")] circle: Option<graphics::Image>,
    #[cfg(feature="graphics")] overlay: Option<graphics::Image>,
}
impl Note {
    pub fn new(
        time: f32,
        hit_type: HitType,
        finisher: bool,
    ) -> Self {

        Self {
            time,
            hit_type,
            base_finisher: finisher,
            finisher,

            ..Default::default()
        }
    }

    pub fn hit(&mut self, time: f32) {
        debug_assert!(!self.missed, "hit a missed note");

        self.hit_time = time;
        self.hit = true;
    }

    pub fn miss(&mut self, time: f32) {
        debug_assert!(!self.hit, "missed a hit note");

        self.hit_time = time;
        self.missed = true;
    }

    pub fn toggle_finishers(&mut self, enabled: bool) {
        self.finisher = self.base_finisher && enabled;
    }

    #[cfg(feature="graphics")]
    pub fn draw(&self, shell: &mut DrawShell) {
        let x = shell.playfield.note_pos(self.time - shell.time, self.speed);

        let hit_delta_time = shell.time - self.hit_time;

        let y = if self.hit {
            GRAVITY_SCALING * 9.81 * (hit_delta_time/1000.0).powi(2) - (hit_delta_time * BOUNCE_VELOCITY)
        } else if self.missed {
            GRAVITY_SCALING * 9.81 * (hit_delta_time/1000.0).powi(2)
        } else { 0.0 };

        let pos = shell.playfield.hit_position + Vector2::new(x, y);

        let radius = if self.finisher {
            shell.settings.note_radius * shell.settings.big_note_multiplier
        } else {
            shell.settings.note_radius
        };

        if pos.x + radius < shell.playfield.pos.x
            || pos.x - radius > shell.playfield.pos.x + shell.playfield.size.x
        {
            return;
        }

        if let Some(image) = self.overlay.clone() {
            let transform = graphics::Transform {
                origin: image.size() / 2.0,
                scale: Vector2::ONE * (radius * 2.0) / NOTE_TEX_SIZE,
                pos,
                ..graphics::Transform::identity()
            };

            shell.list.push(image.with_transform(transform.matrix()));
        }

        if let Some(image) = self.circle.clone() {
            let transform = graphics::Transform {
                origin: image.size() / 2.0,
                scale: Vector2::ONE * (radius * 2.0) / NOTE_TEX_SIZE,
                pos,
                ..graphics::Transform::identity()
            };

            shell.list.push(image.with_transform(transform.matrix()));
        } else {
            let transform = graphics::Transform {
                scale: Vector2::ONE * radius,
                pos,
                ..graphics::Transform::identity()
            };

            let color = match self.hit_type {
                HitType::Don => shell.settings.don_color.color,
                HitType::Kat => shell.settings.kat_color.color,
            };

            let mut circle = graphics::Circle::new(color);

            if self.overlay.is_none() {
                circle.border = Some(Border::new(
                    Color::BLACK,
                    NOTE_BORDER_SIZE
                ));
            }

            shell.list.push(circle.with_transform(transform.matrix()));
        }
    }

    pub fn reset(&mut self) {
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        let hitcircle = if self.finisher {
            "taikobigcircle"
        } else {
            "taikohitcircle"
        };

        let overlay_name = format!("{hitcircle}overlay");
        self.overlay = skin_manager.get_texture(
            Path::new(&overlay_name),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        );

        self.circle = skin_manager.get_texture(
            Path::new(hitcircle),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        );
    }
}
