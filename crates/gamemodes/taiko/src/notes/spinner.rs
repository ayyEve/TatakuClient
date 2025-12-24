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
const SPINNER_RADIUS:f32 = 200.0;

#[derive(Clone, Default)]
pub struct TaikoSpinner {
    hit_count: u16,
    complete: bool, // is this spinner done
    last_hit: Option<HitType>,

    time: f32, // ms
    end_time: f32, // ms
    hits_required: u16, // how many hits until the spinner is "done"
    
    settings: Arc<TaikoSettings>,
    
    #[cfg(feature="graphics")] speed: f32,
    #[cfg(feature="graphics")] pos: Vector2, // the note in the bar, not the spinner itself
    #[cfg(feature="graphics")] don_color: Color,
    #[cfg(feature="graphics")] kat_color: Color,
    #[cfg(feature="graphics")] spinner_image: Option<graphics::Image>,
    #[cfg(feature="graphics")] playfield: Arc<TaikoPlayfield>,
}
impl TaikoSpinner {
    pub fn new(
        time: f32, 
        end_time: f32, 
        hits_required: u16, 
        settings: Arc<TaikoSettings>, 
        #[cfg(feature="graphics")] playfield: Arc<TaikoPlayfield>
    ) -> Self {
        Self {
            time, 
            end_time,
            hits_required,
            
            #[cfg(feature="graphics")] playfield,
            #[cfg(feature = "graphics")] don_color: settings.don_color.color,
            #[cfg(feature = "graphics")] kat_color: settings.kat_color.color,
            settings,

            ..Default::default()
        }
    }
}
impl HitObject for TaikoSpinner {
    fn note_type(&self) -> NoteType { NoteType::Spinner }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 {
        // if the spinner is done, end right away
        if self.complete { self.time } else { self.end_time }
    }

    fn update(&mut self, beatmap_time: f32) {
        if beatmap_time > self.end_time { self.complete = true }
    }

    #[cfg(feature="graphics")]
    fn draw(&mut self, time: f32, list: &mut graphics::RenderableCollection) {
        // if done, dont draw anything
        if self.complete { return }
        self.pos = self.playfield.hit_position + Vector2::with_x(self.x_at(time));

        let spinner_position = self.playfield.hit_position + Vector2::new(100.0, 0.0);

        // if its time to start hitting the spinner
        if self.pos.x <= self.playfield.hit_position.x {
            let mut transform = graphics::Transform {
                pos: spinner_position,
                scale: Vector2::ONE * SPINNER_RADIUS,
                ..graphics::Transform::identity()
            };

            // bg circle
            list.push(graphics::Circle::new(
                Color::YELLOW
            ).border(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            .with_transform(transform.matrix()));

            // draw another circle on top which increases in radius as the counter gets closer to the reqired
            transform.scale *= self.hit_count as f32 / self.hits_required as f32;
            list.push(graphics::Circle::new(
                Color::WHITE,
            ).border(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            .with_transform(transform.matrix()));
            
            //TODO: draw a counter

        } else { // just draw the note on the playfield
            
            if self.pos.x + self.settings.note_radius < self.playfield.pos.x || self.pos.x - self.settings.note_radius > self.playfield.pos.x + self.playfield.size.x { return }
            if let Some(image) = &self.spinner_image {
                let transform = graphics::Transform {
                    pos: self.pos,
                    ..graphics::Transform::identity()
                };

                list.push(image.clone().with_transform(transform.matrix()));
            } else {
                let transform = graphics::Transform {
                    pos: self.pos,
                    scale: Vector2::ONE * self.settings.note_radius,
                    ..graphics::Transform::identity()
                };

                list.push(graphics::HalfCircle::new(
                    self.don_color,
                    true
                ).with_transform(transform.matrix()));

                list.push(graphics::HalfCircle::new(
                    self.kat_color,
                    false
                ).with_transform(transform.matrix()));
            }
        }
    }

    fn reset(&mut self) {
        self.hit_count = 0;
        self.complete = false;
        
        #[cfg(feature="graphics")] {
            self.pos.x = 0.0;
        }
    }
    
    #[cfg(feature="graphics")]
    fn reload_skin(
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
impl TaikoHitObject for TaikoSpinner {
    fn force_hit(&mut self) { self.complete = true }
    fn was_hit(&self) -> bool { self.complete }
    fn is_kat(&self) -> bool { self.last_hit == Some(HitType::Kat) }
    fn hits_to_complete(&self) -> u32 { self.hits_required as u32 }

    // if the spinner wasnt completed in time, cause a miss
    fn causes_miss(&self) -> bool { !self.complete } 

    fn hit(&mut self, time: f32, hit_type: HitType) -> bool {
        // too soon or too late
        if time < self.time || time > self.end_time { return false }
        // wrong note, or already done (just in case)
        if self.complete { return false }

        if let Some(last) = self.last_hit {
            if last == hit_type {
                return false;
            }
            self.last_hit = Some(!last);
        } else {
            self.last_hit = Some(hit_type);
        }


        self.hit_count += 1;
        if self.hit_count == self.hits_required { self.complete = true }

        !self.complete
    }


    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings.clone();
    }

    fn set_required_hits(&mut self, required_hits: u16) {
        self.hits_required = required_hits;
    }

    #[cfg(feature="graphics")] 
    fn get_sv(&self) -> f32 { self.speed }

    #[cfg(feature="graphics")] 
    fn set_sv(&mut self, sv: f32) { self.speed = sv }

    #[cfg(feature="graphics")] 
    fn playfield_changed(&mut self, new_playfield: Arc<TaikoPlayfield>) {
        self.playfield = new_playfield;
    }
    #[cfg(feature="graphics")] 
    fn get_playfield(&self) -> Arc<TaikoPlayfield> {
        self.playfield.clone()
    }
    
}
