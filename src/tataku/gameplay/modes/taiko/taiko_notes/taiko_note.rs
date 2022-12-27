use crate::prelude::*;
use super::super::prelude::*;

#[derive(Clone)]
pub struct TaikoNote {
    pos: Vector2,
    time: f32, // ms
    depth: f64,
    hit_time: f32,
    hit_type: HitType,
    finisher: bool,
    hit: bool,
    missed: bool,
    speed: f32,

    settings: Arc<TaikoSettings>,
    playfield: Arc<TaikoPlayfield>,

    bounce_factor: f32,

    image: Option<HitCircleImageHelper>,
}
impl TaikoNote {
    pub async fn new(time:f32, hit_type:HitType, finisher:bool, settings:Arc<TaikoSettings>, playfield: Arc<TaikoPlayfield>, _diff_calc_only: bool) -> Self {

        // let big_note_radius = settings.note_radius * settings.big_note_multiplier;
        // let y = settings.hit_position.y + big_note_radius * 2.0;
        // let a = GRAVITY_SCALING * 9.81;
        // let bounce_factor = (2000.0*y.sqrt()) as f32 / (a*(a.powi(2) + 2_000_000.0)).sqrt() * 10.0;
        let bounce_factor = 1.6;

        let depth = TaikoGame::get_depth(time);

        Self {
            time, 
            hit_time: 0.0,
            depth,
            hit_type, 
            finisher,
            speed: 0.0,
            hit: false,
            missed: false,
            pos: Vector2::zero(),
            image: None,
            settings,
            playfield,
            bounce_factor
        }
    }

    fn get_color(&mut self) -> Color {
        match self.hit_type {
            HitType::Don => self.settings.don_color,
            HitType::Kat => self.settings.kat_color,
        }
    }
}

#[async_trait]
impl HitObject for TaikoNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}

    async fn update(&mut self, beatmap_time: f32) {

        let delta_time = beatmap_time - self.hit_time;
        let y = 
            if self.hit { GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2) - (delta_time * self.bounce_factor) } 
            else if self.missed { GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2) } 
            else { 0.0 };

        let x = self.x_at(beatmap_time);
        self.pos = self.settings.hit_position + Vector2::new(x as f64, y as f64);

        if let Some(image) = &mut self.image {
            image.set_pos(self.pos)
        }
        
    }
    async fn draw(&mut self, args:RenderArgs, list: &mut RenderableCollection) {
        if self.pos.x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > args.window_size[0] as f64 { return }

        if let Some(image) = &mut self.image {
            image.draw(list);
        } else {
            list.push(Circle::new(
                self.get_color(),
                self.depth,
                self.pos,
                if self.finisher {self.settings.note_radius * self.settings.big_note_multiplier} else {self.settings.note_radius},
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            ));
        }
    }

    async fn reset(&mut self) {
        self.pos = Vector2::zero();
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
    }

    async fn reload_skin(&mut self) {
        self.image = HitCircleImageHelper::new(&self.settings, self.depth, self.hit_type, self.finisher).await;
    }
}
impl TaikoHitObject for TaikoNote {
    fn was_hit(&self) -> bool { self.hit || self.missed }
    fn force_hit(&mut self) { self.hit = true }
    fn get_sv(&self) -> f32 { self.speed }
    fn set_sv(&mut self, sv:f32) { self.speed = sv }
    fn is_kat(&self) -> bool { self.hit_type == HitType::Kat }
    fn finisher_sound(&self) -> bool { self.finisher }
    fn causes_miss(&self) -> bool { true }

    fn hit(&mut self, time: f32) -> bool {
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


    fn playfield_changed(&mut self, new_playfield: Arc<TaikoPlayfield>) {
        self.playfield = new_playfield
    }
    fn get_playfield(&self) -> Arc<TaikoPlayfield> {
        self.playfield.clone()
    }
    
    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings.clone();
        if let Some(i) = &mut self.image {
            i.update_settings(settings, self.finisher)
        }
    }
}

