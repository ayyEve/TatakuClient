use super::*;
use crate::prelude::*;

const SLIDER_DOT_RADIUS:f64 = 8.0;
const SPINNER_RADIUS:f64 = 200.0;
pub const FINISHER_LENIENCY:f32 = 20.0; // ms
const NOTE_BORDER_SIZE:f64 = 2.0;

const GRAVITY_SCALING:f32 = 400.0;

const NOTE_DEPTH_RANGE:std::ops::Range<f64> = 0.0..1000.0;


#[inline]
fn get_depth(time: f32) -> f64 {
    NOTE_DEPTH_RANGE.start + (NOTE_DEPTH_RANGE.end - NOTE_DEPTH_RANGE.end / time as f64)
}


pub trait TaikoHitObject: HitObject + Send + Sync {
    fn is_kat(&self) -> bool { false } // needed for diff calc and autoplay

    fn get_sv(&self) -> f32;
    fn set_sv(&mut self, sv:f32);
    /// does this hit object play a finisher sound when hit?
    fn finisher_sound(&self) -> bool { false }

    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool;
    
    // fn get_points(&mut self, hit_type:HitType, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit;

    /// returns true if a finisher was successfully hit
    fn check_finisher(&mut self, _hit_type:HitType, _time:f32, _game_speed: f32) -> bool { false }

    fn get_playfield(&self) -> Arc<TaikoPlayfield>;
    fn set_settings(&mut self, settings: Arc<TaikoSettings>);


    fn x_at(&self, time:f32) -> f32 {
        // (self.time() - time) * self.get_sv()
        ((self.time() - time) / SV_OVERRIDE) * self.get_sv() * self.get_playfield().size.x as f32
    }
    fn end_x_at(&self, time:f32) -> f32 {
        ((self.end_time(0.0) - time) / SV_OVERRIDE) * self.get_sv() * self.get_playfield().size.x as f32
    }

    fn time_at(&self, x: f32) -> f32 {
        -(x / self.get_sv()) + self.time()
    }

    fn hit_type(&self) -> HitType {
        if self.is_kat() { HitType::Kat } else { HitType::Don }
    }
    
    fn was_hit(&self) -> bool;
    fn force_hit(&mut self) {}

    fn hit(&mut self, _time: f32) -> bool { false }
    fn miss(&mut self, _time: f32) {}

    fn hits_to_complete(&self) -> u32 { 1 }

    fn playfield_changed(&mut self, _new_playfield: Arc<TaikoPlayfield>);
}


// note
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

        let depth = get_depth(time);

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
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        if self.pos.x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > args.window_size[0] as f64 {return list}

        if let Some(image) = &mut self.image {
            image.draw(&mut list);
        } else {
            list.push(Box::new(Circle::new(
                self.get_color(),
                self.depth,
                self.pos,
                if self.finisher {self.settings.note_radius * self.settings.big_note_multiplier} else {self.settings.note_radius},
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));
        }

        list
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

    fn check_finisher(&mut self, hit_type:HitType, time:f32, game_speed: f32) -> bool {
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
        if let Some(i) = &mut self.image{
            i.update_settings(settings, self.finisher)
        }
    }
}


// slider
#[derive(Clone)]
pub struct TaikoSlider {
    pos: Vector2,
    hit_dots: Vec<f32>, // list of times the slider was hit at

    time: f32, // ms
    end_time: f32, // ms
    current_time: f32, 
    finisher: bool,
    speed: f32,
    radius: f64,
    // TODO: figure out how to pre-calc this
    end_x: f64,

    depth: f64,
    settings: Arc<TaikoSettings>,
    playfield: Arc<TaikoPlayfield>,

    middle_image: Option<Image>,
    end_image: Option<Image>,
}
impl TaikoSlider {
    pub async fn new(time:f32, end_time:f32, finisher:bool, settings:Arc<TaikoSettings>, playfield: Arc<TaikoPlayfield>, _diff_calc_only: bool) -> Self {
        let radius = if finisher { settings.note_radius * settings.big_note_multiplier } else { settings.note_radius };
        let depth = get_depth(time);

        let middle_image = None;
        let end_image = None;

        Self {
            time, 
            end_time,
            current_time: 0.0,
            finisher,
            radius,
            speed: 0.0,
            depth,

            pos: Vector2::new(0.0,settings.hit_position.y - radius),
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
impl HitObject for TaikoSlider {
    fn note_type(&self) -> NoteType { NoteType::Slider }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 { self.end_time }
    async fn update(&mut self, beatmap_time: f32) {
        self.pos.x = self.settings.hit_position.x + self.x_at(beatmap_time) as f64;
        self.end_x = self.settings.hit_position.x + self.end_x_at(beatmap_time) as f64;
        self.current_time = beatmap_time;
    }
    async fn draw(&mut self, args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();

        if self.end_x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > args.window_size[0] as f64 {return list}

        let color = Color::YELLOW;
        let border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));

        // middle segment
        if let Some(image) = &self.middle_image {
            let mut image = image.clone();
            image.current_pos = self.pos + Vector2::y_only(self.radius);
            image.current_scale.x = self.end_x - self.pos.x;
            list.push(Box::new(image));
        } else {
            // middle
            list.push(Box::new(Rectangle::new(
                color,
                self.depth,
                self.pos,
                Vector2::new(self.end_x - self.pos.x, self.radius * 2.0),
                border.clone()
            )));
        }

        // start + end circles
        if let Some(image) = &self.end_image {
            // start
            let mut start = image.clone();
            start.current_pos = self.pos + Vector2::new(0.0, self.radius);
            start.current_scale.x *= -1.0;
            list.push(Box::new(start));

            // end
            let mut end = image.clone();
            end.current_pos = Vector2::new(self.end_x, self.pos.y + self.radius);
            list.push(Box::new(end));
            
        } else {
            // start circle
            list.push(Box::new(Circle::new(
                color,
                self.depth,
                self.pos + Vector2::new(0.0, self.radius),
                self.radius,
                border.clone()
            )));
            
            // end circle
            list.push(Box::new(Circle::new(
                color,
                self.depth,
                Vector2::new(self.end_x, self.pos.y + self.radius),
                self.radius,
                border.clone()
            )));
        }


        // draw hit dots
        for time in self.hit_dots.iter() {
            let bounce_factor = 1.6;

            let x = self.settings.hit_position.x as f32 + ((time - self.current_time) / SV_OVERRIDE) * self.get_sv() * self.get_playfield().size.x as f32;
            let diff = self.current_time - time;
            let y = self.settings.hit_position.y as f32 + GRAVITY_SCALING * 9.81 * (diff/1000.0).powi(2) - (diff * bounce_factor);

            // flying dot
            list.push(Box::new(Circle::new(
                Color::YELLOW,
                -1.0,
                Vector2::new(x as f64, y as f64),
                SLIDER_DOT_RADIUS,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE/2.0))
            )));

            // "hole"
            list.push(Box::new(Circle::new(
                BAR_COLOR,
                -1.0,
                Vector2::new(x as f64, self.pos.y + self.radius),
                SLIDER_DOT_RADIUS,
                None
            )))
        }

        list
    }

    async fn reset(&mut self) {
        self.hit_dots.clear();
        self.pos.x = 0.0;
        self.end_x = 0.0;
    }
    
    async fn reload_skin(&mut self) {
        let mut middle_image = SkinManager::get_texture("taiko-roll-middle", true).await;
        if let Some(image) = &mut middle_image {
            image.depth = self.depth;
            image.origin.x = 0.0;
            image.current_color = Color::YELLOW;

            let radius = self.settings.note_radius * if self.finisher {self.settings.big_note_multiplier} else {1.0};
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
            image.initial_scale = scale;
            image.current_scale = scale;
        }
        self.middle_image = middle_image;

        let mut end_image = SkinManager::get_texture("taiko-roll-end", true).await;
        if let Some(image) = &mut end_image {
            image.depth = self.depth;
            image.origin.x = 0.0;
            image.current_color = Color::YELLOW;

            let radius = self.settings.note_radius * if self.finisher {self.settings.big_note_multiplier} else {1.0};
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
            image.initial_scale = scale;
            image.current_scale = scale;
        }
        self.end_image = end_image;

    }
}
impl TaikoHitObject for TaikoSlider {
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
        self.playfield = new_playfield
    }
    fn get_playfield(&self) -> Arc<TaikoPlayfield> {
        self.playfield.clone()
    }
    
    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings.clone();

        for i in [&mut self.middle_image, &mut self.end_image] {
            if let Some(i) = i {
                let radius = settings.note_radius * if self.finisher {settings.big_note_multiplier} else {1.0};
                let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
                i.initial_scale = scale;
                i.current_scale = scale;
            }
        }
    }
}


// spinner
#[derive(Clone)]
pub struct TaikoSpinner {
    pos: Vector2, // the note in the bar, not the spinner itself
    hit_count: u16,
    complete: bool, // is this spinner done

    hits_required: u16, // how many hits until the spinner is "done"
    time: f32, // ms
    end_time: f32, // ms
    speed: f32,

    depth: f64,
    settings: Arc<TaikoSettings>,
    playfield: Arc<TaikoPlayfield>,

    spinner_image: Option<Image>,

    don_color: Color,
    kat_color: Color,
}
impl TaikoSpinner {
    pub async fn new(time:f32, end_time:f32, hits_required:u16, settings:Arc<TaikoSettings>, playfield: Arc<TaikoPlayfield>, _diff_calc_only: bool) -> Self {
        let don_color = settings.don_color;
        let kat_color = settings.kat_color;
        let depth = get_depth(time);

        Self {
            time, 
            end_time,
            speed: 0.0,
            hits_required,
            depth,

            hit_count: 0,
            complete: false,
            pos: Vector2::zero(),

            settings,
            playfield,
            
            spinner_image: None,
            don_color,
            kat_color
        }
    }
}

#[async_trait]
impl HitObject for TaikoSpinner {
    fn note_type(&self) -> NoteType { NoteType::Spinner }
    fn time(&self) -> f32 { self.time }
    fn end_time(&self,_:f32) -> f32 {
        // if the spinner is done, end right away
        if self.complete { self.time } else { self.end_time }
    }

    async fn update(&mut self, beatmap_time: f32) {
        self.pos = self.settings.hit_position + Vector2::new(self.x_at(beatmap_time) as f64, 0.0);
        if beatmap_time > self.end_time { self.complete = true }
    }
    async fn draw(&mut self, _args:RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();

        // if done, dont draw anything
        if self.complete { return list }

        let spinner_position = Vector2::new(self.settings.hit_position.x + 100.0, self.settings.hit_position.y + 0.0);

        // if its time to start hitting the spinner
        if self.pos.x <= self.settings.hit_position.x {
            // bg circle
            list.push(Box::new(Circle::new(
                Color::YELLOW,
                -5.0,
                spinner_position,
                SPINNER_RADIUS,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));

            // draw another circle on top which increases in radius as the counter gets closer to the reqired
            list.push(Box::new(Circle::new(
                Color::WHITE,
                -5.0,
                spinner_position,
                SPINNER_RADIUS * (self.hit_count as f64 / self.hits_required as f64),
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));
            
            //TODO: draw a counter

        } else { // just draw the note on the playfield
            if let Some(image) = &self.spinner_image {
                let mut i = image.clone();
                i.current_pos = self.pos;
                list.push(Box::new(i));
            } else {
                list.push(Box::new(HalfCircle::new(
                    self.don_color,
                    self.pos,
                    self.depth,
                    self.settings.note_radius,
                    true
                )));

                list.push(Box::new(HalfCircle::new(
                    self.kat_color,
                    self.pos,
                    self.depth,
                    self.settings.note_radius,
                    false
                )));
            }
        }

        list
    }

    async fn reset(&mut self) {
        self.pos.x = 0.0;
        self.hit_count = 0;
        self.complete = false;
    }
    
    async fn reload_skin(&mut self) {
        let mut spinner_image = SkinManager::get_texture("spinner-warning", true).await;
        
        if let Some(image) = &mut spinner_image {
            image.depth = self.depth;
        }

        self.spinner_image = spinner_image;
    }
}
impl TaikoHitObject for TaikoSpinner {
    fn force_hit(&mut self) {self.complete = true}
    fn was_hit(&self) -> bool {self.complete}
    fn get_sv(&self) -> f32 {self.speed}
    fn set_sv(&mut self, sv:f32) {self.speed = sv}
    fn is_kat(&self) -> bool { false }
    fn hits_to_complete(&self) -> u32 { self.hits_required as u32 }

    fn causes_miss(&self) -> bool {!self.complete} // if the spinner wasnt completed in time, cause a miss

    fn hit(&mut self, time: f32) -> bool {
        // too soon or too late
        if time < self.time || time > self.end_time { return false }
        // wrong note, or already done (just in case)
        if self.complete { return false }

        self.hit_count += 1;
        if self.hit_count == self.hits_required { self.complete = true }

        !self.complete
    }


    fn playfield_changed(&mut self, new_playfield: Arc<TaikoPlayfield>) {
        self.playfield = new_playfield
    }
    fn get_playfield(&self) -> Arc<TaikoPlayfield> {
        self.playfield.clone()
    }
    
    fn set_settings(&mut self, settings: Arc<TaikoSettings>) {
        self.settings = settings.clone();

        if let Some(i) = &mut self.spinner_image {
            let radius = settings.note_radius;
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
            i.initial_scale = scale;
            i.current_scale = scale;
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitType {
    Don,
    Kat
}
impl Into<HitType> for KeyPress {
    fn into(self) -> HitType {
        match self {
            KeyPress::LeftKat|KeyPress::RightKat => HitType::Kat,
            KeyPress::LeftDon|KeyPress::RightDon => HitType::Don,
            _ => { panic!("non-taiko key while playing taiko") }
        }
    }
}



#[derive(Clone)]
struct HitCircleImageHelper {
    circle: Image,
    overlay: Image,
}
impl HitCircleImageHelper {
    async fn new(settings: &Arc<TaikoSettings>, depth: f64, hit_type: HitType, finisher: bool) -> Option<Self> {
        let color = match hit_type {
            HitType::Don => settings.don_color,
            HitType::Kat => settings.kat_color,
        };


        let radius;
        let hitcircle = if finisher {
            radius = settings.note_radius * settings.big_note_multiplier;
            "taikobigcircle"
        } else {
            radius = settings.note_radius;
            "taikohitcircle"
        };


        let mut circle = SkinManager::get_texture(hitcircle, true).await;
        if let Some(circle) = &mut circle {
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;

            circle.depth = depth;
            circle.initial_pos = Vector2::zero();
            circle.initial_scale = scale;
            circle.initial_color = color;
            
            circle.current_pos = circle.initial_pos;
            circle.current_scale = circle.initial_scale;
            circle.current_color = circle.initial_color;
        }

        let mut overlay = SkinManager::get_texture(hitcircle.to_owned() + "overlay", true).await;
        if let Some(overlay) = &mut overlay {
            let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;

            overlay.depth = depth - 0.0000001;
            overlay.initial_pos = Vector2::zero();
            overlay.initial_scale = scale;
            overlay.initial_color = color;
            
            overlay.current_pos = overlay.initial_pos;
            overlay.current_scale = overlay.initial_scale;
            overlay.current_color = overlay.initial_color;
        }

        if overlay.is_none() || circle.is_none() {return None}

        Some(Self {
            circle: circle.unwrap(),
            overlay: overlay.unwrap(),
        })
    }

    fn set_pos(&mut self, pos: Vector2) {
        self.circle.current_pos  = pos;
        self.overlay.current_pos = pos;
    }
    fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        list.push(Box::new(self.circle.clone()));
        list.push(Box::new(self.overlay.clone()));
    }

    fn update_settings(&mut self, settings: Arc<TaikoSettings>, finisher: bool) {
        let radius = if finisher {
            settings.note_radius * settings.big_note_multiplier
        } else {
            settings.note_radius
        };

        let scale = Vector2::one() * (radius * 2.0) / TAIKO_NOTE_TEX_SIZE;
        
        self.circle.initial_scale = scale;
        self.circle.current_scale = self.circle.initial_scale;
        
        self.overlay.initial_scale = scale;
        self.overlay.current_scale = self.overlay.initial_scale;
    }
}
