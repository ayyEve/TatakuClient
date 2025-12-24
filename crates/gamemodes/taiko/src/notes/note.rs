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

#[derive(Default, Clone)]
pub struct TaikoNote {
    time: f32, // ms
    hit_time: f32,
    hit_type: HitType,
    base_finisher: bool,
    finisher: bool,
    hit: bool,
    missed: bool,

    settings: Arc<TaikoSettings>,

    #[cfg(feature="graphics")] speed: f32,
    #[cfg(feature="graphics")] pos: Vector2,
    #[cfg(feature="graphics")] bounce_factor: f32,
    #[cfg(feature="graphics")] playfield: Arc<TaikoPlayfield>,

    #[cfg(feature="graphics")] circle: Option<graphics::Image>,
    #[cfg(feature="graphics")] overlay: Option<graphics::Image>,
}
impl TaikoNote {
    pub fn new(
        time: f32,
        hit_type: HitType, 
        finisher: bool, 
        settings: Arc<TaikoSettings>,
        #[cfg(feature="graphics")] playfield: Arc<TaikoPlayfield>
    ) -> Self {

        Self {
            time,
            hit_type,
            base_finisher: finisher,
            finisher,
            settings,
            #[cfg(feature="graphics")] playfield,
            #[cfg(feature="graphics")] bounce_factor: 1.6,

            ..Default::default()
        }
    }


    #[cfg(feature = "graphics")]
    fn get_color(&mut self) -> Color {
        match self.hit_type {
            HitType::Don => self.settings.don_color.color,
            HitType::Kat => self.settings.kat_color.color,
        }
    }
}
impl HitObject for TaikoNote {
    fn note_type(&self) -> NoteType { NoteType::Note }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self, hw_miss: f32) -> f32 { self.time + hw_miss }

    fn update(&mut self, _time: f32) {}

    #[cfg(feature="graphics")]
    fn draw(&mut self, time: f32, list: &mut graphics::RenderableCollection) {
        let x = self.x_at(time);
        let delta_time = time - self.hit_time;
        let y = if self.hit {
            GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2) - (delta_time * self.bounce_factor)
        } else if self.missed {
            GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2)
        } else { 0.0 };

        self.pos = self.playfield.hit_position + Vector2::new(x, y);

        if self.pos.x + self.settings.note_radius < self.playfield.pos.x
            || self.pos.x - self.settings.note_radius > self.playfield.pos.x + self.playfield.size.x
        {
            return
        }

        let radius = if self.finisher {
            self.settings.note_radius * self.settings.big_note_multiplier
        } else {
            self.settings.note_radius
        };

        if let Some(image) = &self.overlay {
            let transform = graphics::Transform {
                pos: self.pos,
                scale: Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE,
                ..graphics::Transform::identity()
            };

            list.push(image.clone().with_transform(transform.matrix()));
        }

        if let Some(image) = &self.circle {
            let transform = graphics::Transform {
                pos: self.pos,
                scale: Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE,
                ..graphics::Transform::identity()
            };

            list.push(image.clone().with_transform(transform.matrix()));
        } else {
            let transform = graphics::Transform {
                pos: self.pos,
                scale: Vector2::ONE * radius,
                ..graphics::Transform::identity()
            };

            let mut circle = graphics::Circle::new(self.get_color());

            if self.overlay.is_none() {
                circle.border = Some(Border::new(
                    Color::BLACK,
                    NOTE_BORDER_SIZE
                ));
            }

            list.push(circle.with_transform(transform.matrix()));
        }
    }

    fn reset(&mut self) {
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;

        #[cfg(feature="graphics")] {
            self.pos = Vector2::ZERO;
        }
    }

    #[cfg(feature="graphics")]
    fn reload_skin(
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
impl TaikoHitObject for TaikoNote {
    fn was_hit(&self) -> bool { self.hit || self.missed }
    fn force_hit(&mut self) { self.hit = true }
    fn is_kat(&self) -> bool { self.hit_type == HitType::Kat }
    fn is_finisher(&self) -> bool { self.finisher }
    fn causes_miss(&self) -> bool { true }

    fn hit(&mut self, time: f32, _: HitType) -> bool {
        self.hit_time = time;
        self.hit = true;
        true
    }
    fn miss(&mut self, time: f32) {
        self.hit_time = time;
        self.missed = true;
    }

    fn check_finisher(&self, hit_type:HitType, time:f32, game_speed: f32) -> bool {
        self.finisher && hit_type == self.hit_type && (time - self.hit_time) < FINISHER_LENIENCY * game_speed
    }

    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings;
    }

    fn toggle_finishers(&mut self, enabled: bool) {
        self.finisher = self.base_finisher && enabled;
        self.set_settings(self.settings.clone());
    }

    #[cfg(feature="graphics")] fn get_sv(&self) -> f32 { self.speed }
    #[cfg(feature="graphics")] fn set_sv(&mut self, sv:f32) { self.speed = sv }
    #[cfg(feature="gameplay")] fn finisher_sound(&self) -> bool { self.base_finisher }
    #[cfg(feature="graphics")]
    fn playfield_changed(&mut self, new_playfield: Arc<TaikoPlayfield>) {
        self.playfield = new_playfield;
    }
    #[cfg(feature="graphics")]
    fn get_playfield(&self) -> Arc<TaikoPlayfield> {
        self.playfield.clone()
    }

}
