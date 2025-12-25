use crate::prelude::*;
use tataku::{
    Color,
    Border,
    Vector2,
};
use engine::{
    beatmaps::NoteType,
    gameplay::HitObject,
};

#[cfg(feature="graphics")]
use engine::graphics;

#[cfg(feature = "graphics")]
const SLIDER_DOT_RADIUS: f32 = 8.0;

#[derive(Clone, Default)]
pub struct Drumroll {
    time: f32, // ms
    end_time: f32, // ms
    /// should this be a finisher
    base_finisher: bool,
    finisher: bool,
    settings: Arc<Settings>,

    #[cfg(feature="graphics")] speed: f32,
    #[cfg(feature="graphics")] end_x: f32,
    #[cfg(feature="graphics")] radius: f32,
    #[cfg(feature="graphics")] pos: Vector2,
    #[cfg(feature="graphics")] hit_dots: Vec<f32>, // list of times the slider was hit at
    #[cfg(feature="graphics")] end_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] middle_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] playfield: Arc<Playfield>,
}
impl Drumroll {
    pub fn new(
        time: f32,
        end_time: f32,
        finisher: bool,
        settings: Arc<Settings>,
        #[cfg(feature="graphics")] playfield: Arc<Playfield>
    ) -> Self {
        #[cfg(feature="graphics")]
        let radius = if finisher {
            settings.note_radius * settings.big_note_multiplier
        } else {
            settings.note_radius
        };

        Self {
            time,
            end_time,
            settings,
            finisher,
            base_finisher: finisher,

            #[cfg(feature="graphics")] radius,
            #[cfg(feature="graphics")] pos: Vector2::new(0.0, playfield.hit_position.y - radius),
            #[cfg(feature="graphics")] playfield,

            ..Default::default()
        }
    }
}
impl HitObject for Drumroll {
    fn note_type(&self) -> NoteType { NoteType::Slider }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 { self.end_time }
    fn update(&mut self, _time: f32) {}

    #[cfg(feature="graphics")]
    fn draw(&mut self, time: f32, list: &mut graphics::RenderableCollection) {
        self.pos.x = self.playfield.hit_position.x + self.x_at(time);
        self.end_x = self.playfield.hit_position.x + self.end_x_at(time);

        if self.end_x + self.settings.note_radius < self.playfield.pos.x
        || self.pos.x - self.settings.note_radius > self.playfield.pos.x + self.playfield.size.x { return }

        let color = Color::YELLOW;
        let border = Border::new(Color::BLACK, NOTE_BORDER_SIZE);

        // middle segment
        if let Some(image) = &self.middle_image {
            let scale = (self.end_x - self.pos.x) / image.size().x;

            let transform = graphics::Transform {
                pos: self.pos + Vector2::with_y(self.radius),
                scale: Vector2::new(scale, 1.0),
                ..graphics::Transform::identity()
            };

            list.push(image.clone().with_transform(transform.matrix()));
        } else {
            // middle
            list.push(graphics::Rectangle::new(
                Vector2::new(self.end_x - self.pos.x, self.radius * 2.0),
                color,
            ).border(border)
            .with_transform(tataku::Matrix::identity()
                .trans(self.pos)
            ));
        }

        // start + end circles
        if let Some(image) = &self.end_image {
            // start
            list.push(image.clone().with_transform(graphics::Transform {
                pos: self.pos + Vector2::new(0.0, self.radius),
                scale: Vector2::new(-1.0, 1.0),
                ..graphics::Transform::identity()
            }.matrix()));

            // end
            list.push(image.clone().with_transform(graphics::Transform {
                pos: Vector2::new(self.end_x, self.pos.y + self.radius),
                ..graphics::Transform::identity()
            }.matrix()));

        } else {
            // start circle
            list.push(graphics::Circle::new(
                color,
            ).border(border)
            .with_transform(graphics::Transform {
                pos: self.pos + Vector2::new(0.0, self.radius),
                scale: Vector2::ONE * self.radius,
                ..graphics::Transform::identity()
            }.matrix()));

            // end circle
            list.push(graphics::Circle::new(
                color,
            ).border(border)
            .with_transform(graphics::Transform {
                pos: Vector2::new(self.end_x, self.pos.y + self.radius),
                scale: Vector2::ONE * self.radius,
                ..graphics::Transform::identity()
            }.matrix()));
        }


        // draw hit dots
        for dot_time in self.hit_dots.iter() {
            let bounce_factor = 1.6;

            let x = self.playfield.hit_position.x + ((dot_time - time) / SV_OVERRIDE) * self.get_sv() * self.playfield.size.x;
            let diff = time - dot_time;
            let y = self.playfield.hit_position.y + GRAVITY_SCALING * 9.81 * (diff/1000.0).powi(2) - (diff * bounce_factor);

            // flying dot
            list.push(graphics::Circle::new(
                Color::YELLOW,
            ).border(Border::new(
                Color::BLACK,
                NOTE_BORDER_SIZE/2.0
            )).with_transform(graphics::Transform {
                pos: Vector2::new(x, y),
                scale: Vector2::ONE * SLIDER_DOT_RADIUS,
                ..graphics::Transform::identity()
            }.matrix()));

            // "hole"
            list.push(graphics::Circle::new(
                BAR_COLOR,
            ).with_transform(graphics::Transform {
                pos: Vector2::new(x, self.pos.y + self.radius),
                scale: Vector2::ONE * SLIDER_DOT_RADIUS,
                ..graphics::Transform::identity()
            }.matrix()));
        }
    }

    fn reset(&mut self) {
        #[cfg(feature="graphics")]  {
            self.hit_dots.clear();
            self.pos.x = 0.0;
            self.end_x = 0.0;
        }
    }

    #[cfg(feature="graphics")]
    fn reload_skin(
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
impl TaikoHitObject for Drumroll {
    fn was_hit(&self) -> bool { false }
    fn causes_miss(&self) -> bool { false }
    fn hits_to_complete(&self) -> u32 { ((self.end_time - self.time) / 50.0) as u32 }

    fn hit(&mut self, time: f32, _: HitType) -> bool {
        if time < self.time || time > self.end_time { return false }
        #[cfg(feature="graphics")]
        self.hit_dots.push(time);
        true
    }

    fn set_settings(&mut self, settings: Arc<Settings>) {
        self.settings = settings;
    }

    fn toggle_finishers(&mut self, enabled: bool) {
        if self.base_finisher {
            self.finisher = enabled;
            self.set_settings(self.settings.clone());
        }
    }

    #[cfg(feature="graphics")] fn get_sv(&self) -> f32 { self.speed }
    #[cfg(feature="graphics")] fn set_sv(&mut self, sv: f32) { self.speed = sv }
    #[cfg(feature="graphics")]
    fn playfield_changed(&mut self, new_playfield: Arc<Playfield>) {
        self.playfield = new_playfield;
        self.pos.y = self.playfield.hit_position.y - self.radius;
    }
    #[cfg(feature="graphics")]
    fn get_playfield(&self) -> Arc<Playfield> {
        self.playfield.clone()
    }
}
