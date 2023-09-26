use crate::prelude::*;
use super::super::prelude::*;

const SLIDER_DOT_RADIUS:f32 = 8.0;

#[derive(Clone)]
pub struct TaikoDrumroll {
    pos: Vector2,
    hit_dots: Vec<f32>, // list of times the slider was hit at

    time: f32, // ms
    end_time: f32, // ms
    /// should this be a finisher
    pub base_finisher: bool,
    pub finisher: bool,
    speed: f32,
    radius: f32,
    // TODO: figure out how to pre-calc this
    end_x: f32,

    settings: Arc<TaikoSettings>,
    playfield: Arc<TaikoPlayfield>,

    middle_image: Option<Image>,
    end_image: Option<Image>,
}
impl TaikoDrumroll {
    pub async fn new(time:f32, end_time:f32, finisher:bool, settings:Arc<TaikoSettings>, playfield: Arc<TaikoPlayfield>) -> Self {
        let radius = if finisher { settings.note_radius * settings.big_note_multiplier } else { settings.note_radius };

        let middle_image = None;
        let end_image = None;

        Self {
            time, 
            end_time,
            base_finisher: finisher,
            finisher,
            radius,
            speed: 0.0,

            pos: Vector2::new(0.0, playfield.hit_position.y - radius),
            end_x: 0.0,
            hit_dots: Vec::new(),
            settings,
            playfield,

            middle_image,
            end_image
        }
    }
}

#[async_trait]
impl HitObject for TaikoDrumroll {
    fn note_type(&self) -> NoteType { NoteType::Slider }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 { self.end_time }
    async fn update(&mut self, _time: f32) {}
    
    async fn draw(&mut self, time: f32, list: &mut RenderableCollection) {
        self.pos.x = self.playfield.hit_position.x + self.x_at(time);
        self.end_x = self.playfield.hit_position.x + self.end_x_at(time);

        if self.end_x + self.settings.note_radius < self.playfield.pos.x || self.pos.x - self.settings.note_radius > self.playfield.pos.x + self.playfield.size.x { return }

        let color = Color::YELLOW;
        let border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));

        // middle segment
        if let Some(image) = &self.middle_image {
            let mut image = image.clone();
            image.pos = self.pos + Vector2::with_y(self.radius);
            image.scale.x = self.end_x - self.pos.x;
            list.push(image);
        } else {
            // middle
            list.push(Rectangle::new(
                self.pos,
                Vector2::new(self.end_x - self.pos.x, self.radius * 2.0),
                color,
                border.clone()
            ));
        }

        // start + end circles
        if let Some(image) = &self.end_image {
            // start
            let mut start = image.clone();
            start.pos = self.pos + Vector2::new(0.0, self.radius);
            start.scale.x *= -1.0;
            // start.origin.x = start.tex_size().x;
            list.push(start);

            // end
            let mut end = image.clone();
            end.pos = Vector2::new(self.end_x, self.pos.y + self.radius);
            list.push(end);
            
        } else {
            // start circle
            list.push(Circle::new(
                self.pos + Vector2::new(0.0, self.radius),
                self.radius,
                color,
                border.clone()
            ));
            
            // end circle
            list.push(Circle::new(
                Vector2::new(self.end_x, self.pos.y + self.radius),
                self.radius,
                color,
                border.clone()
            ));
        }


        // draw hit dots
        for dot_time in self.hit_dots.iter() {
            let bounce_factor = 1.6;

            let x = self.playfield.hit_position.x + ((dot_time - time) / SV_OVERRIDE) * self.get_sv() * self.playfield.size.x;
            let diff = time - dot_time;
            let y = self.playfield.hit_position.y + GRAVITY_SCALING * 9.81 * (diff/1000.0).powi(2) - (diff * bounce_factor);

            // flying dot
            list.push(Circle::new(
                Vector2::new(x, y),
                SLIDER_DOT_RADIUS,
                Color::YELLOW,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE/2.0))
            ));

            // "hole"
            list.push(Circle::new(
                Vector2::new(x, self.pos.y + self.radius),
                SLIDER_DOT_RADIUS,
                BAR_COLOR,
                None
            ))
        }
    }

    async fn reset(&mut self) {
        self.hit_dots.clear();
        self.pos.x = 0.0;
        self.end_x = 0.0;
    }
    
    async fn reload_skin(&mut self) {
        let mut middle_image = SkinManager::get_texture("taiko-roll-middle", true).await;
        if let Some(image) = &mut middle_image {
            image.origin.x = 0.0;
            image.color = Color::YELLOW;

            let radius = self.settings.note_radius * if self.finisher {self.settings.big_note_multiplier} else {1.0};
            image.scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
        }
        self.middle_image = middle_image;

        let mut end_image = SkinManager::get_texture("taiko-roll-end", true).await;
        if let Some(image) = &mut end_image {
            image.origin.x = 0.0;
            image.color = Color::YELLOW;

            let radius = self.settings.note_radius * if self.finisher {self.settings.big_note_multiplier} else {1.0};
            image.scale = Vector2::ONE * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
        }
        self.end_image = end_image;

    }
}
impl TaikoHitObject for TaikoDrumroll {
    fn was_hit(&self) -> bool { false }
    fn causes_miss(&self) -> bool { false }
    fn get_sv(&self) -> f32 { self.speed }
    fn set_sv(&mut self, sv:f32) { self.speed = sv }
    fn hits_to_complete(&self) -> u32 { ((self.end_time - self.time) / 50.0) as u32 }

    fn hit(&mut self, time: f32) -> bool {
        if time < self.time || time > self.end_time { return false }
        self.hit_dots.push(time);
        true
    }

    fn playfield_changed(&mut self, new_playfield: Arc<TaikoPlayfield>) {
        self.playfield = new_playfield;
        self.pos.y = self.playfield.hit_position.y - self.radius;
    }
    fn get_playfield(&self) -> Arc<TaikoPlayfield> {
        self.playfield.clone()
    }
    
    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings;

        for i in [&mut self.middle_image, &mut self.end_image] {
            if let Some(i) = i {
                // let radius = self.settings.note_radius * if self.finisher {self.settings.big_note_multiplier} else {1.0};
                i.scale = Vector2::ONE * (self.radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
            }
        }
    }
    
    fn toggle_finishers(&mut self, enabled: bool) {
        if self.base_finisher {
            self.finisher = enabled;
            self.set_settings(self.settings.clone());
        }
    }
}
