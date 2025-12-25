use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Vector2,
};

#[cfg(feature="graphics")]
use engine::graphics;

#[cfg(feature = "graphics")]
const SLIDER_DOT_RADIUS: f32 = 8.0;

#[derive(Clone, Default)]
pub struct Drumroll {
    pub time: f32,
    pub end_time: f32,

    pub base_finisher: bool,
    pub finisher: bool,

    #[cfg(feature="graphics")] pub speed: f32,
    #[cfg(feature="graphics")] hit_dots: Vec<f32>, // list of times the slider was hit at
    #[cfg(feature="graphics")] end_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] middle_image: Option<graphics::Image>,
}
impl Drumroll {
    pub fn new(
        time: f32,
        end_time: f32,
        finisher: bool,
    ) -> Self {
        Self {
            time,
            end_time,
            finisher,
            base_finisher: finisher,
            ..Default::default()
        }
    }

    pub fn toggle_finishers(&mut self, enabled: bool) {
        if self.base_finisher {
            self.finisher = enabled;
        }
    }

    pub fn hit(&mut self, time: f32, _: HitType) -> bool {
        if time < self.time || time > self.end_time { return false }
        #[cfg(feature="graphics")]
        self.hit_dots.push(time);
        true
    }

    #[cfg(feature="graphics")]
    pub fn draw(&self, shell: &mut DrawShell) {
        let x = shell.playfield.note_pos(self.time - shell.time, self.speed);
        let end_x = shell.playfield.note_pos(self.end_time - shell.time, self.speed);

        let pos = shell.playfield.hit_position + Vector2::with_x(x);
        let end_pos = shell.playfield.hit_position + Vector2::with_x(end_x);

        let radius = if self.finisher {
            shell.settings.note_radius * shell.settings.big_note_multiplier
        } else {
            shell.settings.note_radius
        };

        if end_pos.x + radius < shell.playfield.pos.x
            || pos.x - radius > shell.playfield.pos.x + shell.playfield.size.x
        {
            return;
        }

        let color = Color::YELLOW;
        let border = Border::new(Color::BLACK, NOTE_BORDER_SIZE);

        // middle segment
        if let Some(image) = self.middle_image.clone() {
            let scale = Vector2::new(
                end_x - x,
                2.0 * radius,
            ) / image.size();

            let transform = graphics::Transform {
                origin: Vector2::with_y(image.size().y), // left-center
                scale,
                pos,
                ..graphics::Transform::identity()
            };

            shell.list.push(image.with_transform(transform.matrix()));
        } else {
            shell.list.push(graphics::Rectangle::new(
                Vector2::new(end_x - x, radius * 2.0),
                color,
            ).border(border)
            .with_transform(tataku::Matrix::identity()
                .trans(pos - Vector2::with_y(radius))
            ));
        }

        // start + end circles
        if let Some(image) = &self.end_image {
            // start
            shell.list.push(image.clone().with_transform(graphics::Transform {
                origin: image.size() / 2.0, // center
                scale: Vector2::new(-1.0, 1.0),
                pos,
                ..graphics::Transform::identity()
            }.matrix()));

            // end
            shell.list.push(image.clone().with_transform(graphics::Transform {
                origin: image.size() / 2.0, // center
                pos: end_pos,
                ..graphics::Transform::identity()
            }.matrix()));

        } else {
            // start circle
            shell.list.push(graphics::Circle::new(color)
                .border(border)
                .with_transform(graphics::Transform {
                    pos,
                    scale: Vector2::ONE * radius,
                    ..graphics::Transform::identity()
                }.matrix())
            );

            // end circle
            shell.list.push(graphics::Circle::new(color)
                .border(border)
                .with_transform(graphics::Transform {
                    pos: end_pos,
                    scale: Vector2::ONE * radius,
                    ..graphics::Transform::identity()
                }.matrix())
            );
        }

        // draw hit dots
        for dot_time in self.hit_dots.iter() {
            let x = shell.playfield.note_pos(dot_time - shell.time, self.speed);

            let delta_time = shell.time - dot_time;
            let y = GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2) - (delta_time * BOUNCE_VELOCITY);

            let dot_pos = shell.playfield.hit_position + Vector2::new(x, y);
            let hole_pos = shell.playfield.hit_position + Vector2::with_x(x);

            // flying dot
            shell.list.push(graphics::Circle::new(Color::YELLOW)
                    .border(Border::new(
                    Color::BLACK,
                    NOTE_BORDER_SIZE/2.0
                )).with_transform(graphics::Transform {
                    pos: dot_pos,
                    scale: Vector2::ONE * SLIDER_DOT_RADIUS,
                    ..graphics::Transform::identity()
                }.matrix())
            );

            // "hole"
            shell.list.push(graphics::Circle::new(BAR_COLOR)
                .with_transform(graphics::Transform {
                    pos: hole_pos,
                    scale: Vector2::ONE * SLIDER_DOT_RADIUS,
                    ..graphics::Transform::identity()
                }.matrix())
            );
        }
    }

    pub fn reset(&mut self) {
        #[cfg(feature="graphics")]
        self.hit_dots.clear();
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(
        &mut self,
        source: &graphics::TextureSource,
        skin_manager: &mut dyn graphics::SkinProvider
    ) {
        self.middle_image = skin_manager.get_texture(
            Path::new("taiko-roll-middle"),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        ).map(|mut i| {
            i.color = Color::YELLOW;
            i
        });

        self.end_image = skin_manager.get_texture(
            Path::new("taiko-roll-end"),
            source,
            graphics::SkinUsage::Gamemode,
            false,
        ).map(|mut i| {
            i.color = Color::YELLOW;
            i
        });
    }
}
