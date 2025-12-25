use crate::prelude::*;

use tataku::{
    Color,
    Border,
    Vector2,
};

#[cfg(feature="graphics")]
use engine::graphics;

#[cfg(feature = "graphics")]
const SPINNER_RADIUS: f32 = 200.0;

#[derive(Clone, Default)]
pub struct Spinner {
    pub hit_count: u16,
    pub last_hit: Option<HitType>,

    pub time: f32,
    pub end_time: f32,
    pub hits_required: u16, // how many hits until the spinner is "done"

    #[cfg(feature="graphics")] pub speed: f32,
    #[cfg(feature="graphics")] spinner_image: Option<graphics::Image>,
}
impl Spinner {
    pub fn new(
        time: f32,
        end_time: f32,
        hits_required: u16,
    ) -> Self {
        Self {
            time,
            end_time,
            hits_required,
            ..Default::default()
        }
    }

    pub fn hit(&mut self, time: f32, hit_type: HitType) -> bool {
        // too soon or too late
        if time < self.time || time > self.end_time { return false }
        // already done (just in case)
        if self.hit_count >= self.hits_required { return false }

        if let Some(last) = self.last_hit {
            if last == hit_type {
                return false;
            }
            self.last_hit = Some(!last);
        } else {
            self.last_hit = Some(hit_type);
        }

        self.hit_count += 1;

        self.hit_count == self.hits_required
    }

    #[cfg(feature="graphics")]
    pub fn draw(&self, shell: &mut DrawShell) {
        // if done, dont draw anything
        if self.hit_count == self.hits_required || shell.time > self.end_time { return; }

        let x = shell.playfield.note_pos(self.time - shell.time, self.speed);

        let pos = shell.playfield.hit_position + Vector2::with_x(x);

        // todo: make customisable
        let spinner_position = shell.playfield.hit_position + Vector2::new(100.0, 0.0);

        // if its time to start hitting the spinner
        if shell.time >= self.time {
            let mut transform = graphics::Transform {
                pos: spinner_position,
                scale: Vector2::ONE * SPINNER_RADIUS,
                ..graphics::Transform::identity()
            };

            // bg circle
            shell.list.push(graphics::Circle::new(Color::YELLOW)
                .border(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
                .with_transform(transform.matrix())
            );

            // draw another circle on top which increases in radius as the counter gets closer to the reqired
            transform.scale *= self.hit_count as f32 / self.hits_required as f32;
            shell.list.push(graphics::Circle::new(Color::WHITE)
                .border(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
                .with_transform(transform.matrix())
            );

            //TODO: draw a counter
        } else {
            // just draw the note on the playfield
            if x + shell.settings.note_radius < shell.playfield.pos.x
                || x - shell.settings.note_radius > shell.playfield.pos.x + shell.playfield.size.x
            {
                return;
            }

            if let Some(image) = self.spinner_image.clone() {
                let transform = graphics::Transform {
                    origin: image.size() / 2.0, // center
                    pos,
                    ..graphics::Transform::identity()
                };

                shell.list.push(image.with_transform(transform.matrix()));
            } else {
                let transform = graphics::Transform {
                    scale: Vector2::ONE * shell.settings.note_radius,
                    pos,
                    ..graphics::Transform::identity()
                };

                shell.list.push(graphics::HalfCircle::new(
                    shell.settings.don_color.color,
                    true
                ).with_transform(transform.matrix()));

                shell.list.push(graphics::HalfCircle::new(
                    shell.settings.kat_color.color,
                    false
                ).with_transform(transform.matrix()));
            }
        }
    }

    pub fn reset(&mut self) {
        self.hit_count = 0;
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        self.spinner_image = skin_manager.get_texture(
            Path::new("spinner-warning"),
            source,
            graphics::SkinUsage::Gamemode,
            false
        );
    }
}
