use crate::prelude::*;
use super::BAR_COLOR;

const SLIDER_DOT_RADIUS:f64 = 8.0;
const SPINNER_RADIUS:f64 = 200.0;
pub const FINISHER_LENIENCY:f32 = 20.0; // ms
const NOTE_BORDER_SIZE:f64 = 2.0;

const GRAVITY_SCALING:f32 = 400.0;

pub trait TaikoHitObject: HitObject {
    fn is_kat(&self) -> bool {false}// needed for diff calc and autoplay

    fn get_sv(&self) -> f32;
    fn set_sv(&mut self, sv:f32);
    /// does this hit object play a finisher sound when hit?
    fn finisher_sound(&self) -> bool {false}

    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    
    fn get_points(&mut self, hit_type:HitType, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit; // if negative, counts as a miss
    fn check_finisher(&mut self, _hit_type:HitType, _time:f32) -> ScoreHit {ScoreHit::None}


    fn x_at(&self, time:f32) -> f32 {(self.time() - time) * self.get_sv()}

    fn was_hit(&self) -> bool;
    fn force_hit(&mut self) {}

    fn hits_to_complete(&self) -> u32 {1}
}


// note
#[derive(Clone)]
pub struct TaikoNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
    hit_type: HitType,
    finisher: bool,
    hit: bool,
    missed: bool,
    speed: f32,

    alpha_mult: f32,
    settings: Arc<TaikoSettings>,

    image: Option<HitCircleImageHelper>
}
impl TaikoNote {
    pub fn new(time:f32, hit_type:HitType, finisher:bool, settings:Arc<TaikoSettings>) -> Self {
        Self {
            time, 
            hit_time: 0.0,
            hit_type, 
            finisher,
            speed: 0.0,
            hit: false,
            missed: false,
            pos: Vector2::zero(),
            alpha_mult: 1.0,
            image: HitCircleImageHelper::new(&settings, time as f64, hit_type, finisher),
            settings,
        }
    }

    fn get_color(&mut self) -> Color {
        match self.hit_type {
            HitType::Don => self.settings.don_color,
            HitType::Kat => self.settings.kat_color,
        }
    }
}
impl HitObject for TaikoNote {
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {
        let delta_time = beatmap_time - self.hit_time;
        let y = 
            if self.hit {GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2) - (delta_time * 1.5)} 
            else if self.missed {GRAVITY_SCALING * 9.81 * (delta_time/1000.0).powi(2)} 
            else {0.0};
        
        self.pos = self.settings.hit_position + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, y as f64);

        if let Some(image) = &mut self.image {
            image.set_pos(self.pos)
        }
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > args.window_size[0] as f64 {return}

        if let Some(image) = &mut self.image {
            image.draw(list);
        } else {
            list.push(Box::new(Circle::new(
                self.get_color().alpha(self.alpha_mult),
                self.time as f64,
                self.pos,
                if self.finisher {self.settings.note_radius * self.settings.big_note_multiplier} else {self.settings.note_radius},
                Some(Border::new(Color::BLACK.alpha(self.alpha_mult), NOTE_BORDER_SIZE))
            )));
        }
    }

    fn reset(&mut self) {
        self.pos = Vector2::zero();
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
    }
}
impl TaikoHitObject for TaikoNote {
    fn was_hit(&self) -> bool {self.hit || self.missed}
    fn force_hit(&mut self) {self.hit = true}
    fn get_sv(&self) -> f32 {self.speed}
    fn set_sv(&mut self, sv:f32) {self.speed = sv}
    fn is_kat(&self) -> bool {self.hit_type == HitType::Kat}
    fn finisher_sound(&self) -> bool {self.finisher}

    fn causes_miss(&self) -> bool {true}

    fn get_points(&mut self, hit_type:HitType, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit {
        let (hitwindow_miss, hitwindow_100, hitwindow_300) = hit_windows;
        let diff = (time - self.time).abs();

        if diff < hitwindow_300 {
            self.hit_time = time.max(0.0);
            if hit_type != self.hit_type {
                self.missed = true;
                ScoreHit::Miss
            } else {
                self.hit = true;
                ScoreHit::X300
            }
        } else if diff < hitwindow_100 {
            self.hit_time = time.max(0.0);
            if hit_type != self.hit_type {
                self.missed = true;
                ScoreHit::Miss
            } else {
                self.hit = true;
                ScoreHit::X100
            }
        } else if diff < hitwindow_miss { // too early, miss
            self.hit_time = time.max(0.0);
            self.missed = true;
            ScoreHit::Miss
        } else { // way too early, ignore
            ScoreHit::None
        }
    }
    fn check_finisher(&mut self, hit_type:HitType, time:f32) -> ScoreHit {
        if self.finisher && hit_type == self.hit_type && (time - self.hit_time) < FINISHER_LENIENCY {
            ScoreHit::X300
        } else {
            ScoreHit::None
        }
    }
}


// slider
#[derive(Clone)]
pub struct TaikoSlider {
    pos: Vector2,
    hit_dots:Vec<SliderDot>, // list of times the slider was hit at

    time: f32, // ms
    end_time: f32, // ms
    finisher: bool,
    speed: f32,
    radius: f64,
    //TODO: figure out how to pre-calc this
    end_x: f64,
    
    alpha_mult: f32,
    settings: Arc<TaikoSettings>,

    middle_image:Option<Image>,
    end_image: Option<Image>,

}
impl TaikoSlider {
    pub fn new(time:f32, end_time:f32, finisher:bool, settings:Arc<TaikoSettings>) -> Self {
        let radius = if finisher {settings.note_radius * settings.big_note_multiplier} else {settings.note_radius};

        let mut middle_image = SKIN_MANAGER.write().get_texture("taiko-roll-middle", true);
        if let Some(image) = &mut middle_image {
            image.depth = time as f64 + 1.0;
            image.origin.x = 0.0;
            image.current_color = Color::YELLOW;
        }

        let mut end_image = SKIN_MANAGER.write().get_texture("taiko-roll-end", true);
        if let Some(image) = &mut end_image {
            image.depth = time as f64;
            image.origin.x = 0.0;
            image.current_color = Color::YELLOW;
        }


        Self {
            time, 
            end_time,
            finisher,
            radius,
            speed: 0.0,

            pos: Vector2::new(0.0,settings.hit_position.y - radius),
            end_x: 0.0,
            hit_dots: Vec::new(),
            
            alpha_mult: 1.0,
            settings,

            middle_image,
            end_image
        }
    }
}
impl HitObject for TaikoSlider {
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.end_time}
    fn update(&mut self, beatmap_time: f32) {
        self.pos.x = self.settings.hit_position.x + ((self.time - beatmap_time) * self.speed) as f64;
        self.end_x = self.settings.hit_position.x + ((self.end_time(0.0) - beatmap_time) * self.speed) as f64;

        // draw hit dots
        for dot in self.hit_dots.iter_mut() {
            if dot.done {continue}
            dot.update(beatmap_time);
        }
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.end_x + self.settings.note_radius < 0.0 || self.pos.x - self.settings.note_radius > args.window_size[0] as f64 {return}

        let color = Color::YELLOW.alpha(self.alpha_mult);
        let border = Some(Border::new(Color::BLACK.alpha(self.alpha_mult), NOTE_BORDER_SIZE));

        if let Some(image) = &self.middle_image {
            let mut image = image.clone();
            image.current_pos = self.pos + Vector2::y_only(self.radius);
            image.current_scale = Vector2::new(self.end_x - self.pos.x, 1.0);
            list.push(Box::new(image));
        } else {
            // middle
            list.push(Box::new(Rectangle::new(
                color,
                self.time as f64 + 1.0,
                self.pos,
                Vector2::new(self.end_x - self.pos.x, self.radius * 2.0),
                border.clone()
            )));
        }

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
                self.time as f64,
                self.pos + Vector2::new(0.0, self.radius),
                self.radius,
                border.clone()
            )));
            
            // end circle
            list.push(Box::new(Circle::new(
                color,
                self.time as f64,
                Vector2::new(self.end_x, self.pos.y + self.radius),
                self.radius,
                border.clone()
            )));
        }


        // draw hit dots
        for dot in self.hit_dots.as_slice() {
            if dot.done {continue}
            dot.draw(list);
        }
    }

    fn reset(&mut self) {
        self.hit_dots.clear();
        self.pos.x = 0.0;
        self.end_x = 0.0;
    }
}
impl TaikoHitObject for TaikoSlider {
    fn was_hit(&self) -> bool {false}
    fn causes_miss(&self) -> bool {false}
    fn get_sv(&self) -> f32 {self.speed}
    fn set_sv(&mut self, sv:f32) {self.speed = sv}
    fn hits_to_complete(&self) -> u32 {((self.end_time - self.time) / 50.0) as u32}

    fn get_points(&mut self, _hit_type:HitType, time:f32, _:(f32,f32,f32)) -> ScoreHit {
        // too soon or too late
        if time < self.time || time > self.end_time {return ScoreHit::None}
        
        self.hit_dots.push(SliderDot::new(time, self.speed, self.settings.clone()));
        ScoreHit::Other(if self.finisher {200} else {100}, false)
    }

}
/// helper struct for drawing hit slider points
#[derive(Clone)]
struct SliderDot {
    time: f32,
    speed: f32,
    pos: Vector2,
    pub done: bool,
    settings: Arc<TaikoSettings>,
}
impl SliderDot {
    pub fn new(time:f32, speed:f32, settings:Arc<TaikoSettings>) -> SliderDot {
        SliderDot {
            time,
            speed,
            pos: Vector2::zero(),
            done: false,
            settings
        }
    }
    pub fn update(&mut self, beatmap_time:f32) {
        let y = -((beatmap_time - self.time)*20.0).ln()*20.0 + 1.0;
        self.pos = self.settings.hit_position + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, y as f64);
        
        if !self.done && self.pos.x - SLIDER_DOT_RADIUS <= 0.0 {
            self.done = true;
        }
    }
    pub fn draw(&self, list: &mut Vec<Box<dyn Renderable>>) {
        println!("drawing dot");
        list.push(Box::new(Circle::new(
            Color::YELLOW,
            -100.0,
            self.pos,
            SLIDER_DOT_RADIUS,
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE/2.0))
        )));
        list.push(Box::new(Circle::new(
            BAR_COLOR,
            0.0,
            Vector2::new(self.pos.x, self.settings.hit_position.y),
            SLIDER_DOT_RADIUS,
            None
        )))
    }
}

// spinner
#[derive(Clone)]
pub struct TaikoSpinner {
    pos: Vector2, // the note in the bar, not the spinner itself
    hit_count: u16,
    last_hit: HitType,
    complete: bool, // is this spinner done

    hits_required: u16, // how many hits until the spinner is "done"
    time: f32, // ms
    end_time: f32, // ms
    speed: f32,

    alpha_mult: f32,
    settings: Arc<TaikoSettings>,

    spinner_image: Option<Image>,

    don_color: Color,
    kat_color: Color,
}
impl TaikoSpinner {
    pub fn new(time:f32, end_time:f32, hits_required:u16, settings:Arc<TaikoSettings>) -> Self {
        let mut spinner_image = SKIN_MANAGER.write().get_texture("spinner-warning", true);

        if let Some(image) = &mut spinner_image {
            image.depth = time as f64;
        }

        let (don_color, kat_color) = {
            let s = &get_settings!().taiko_settings;
            (s.don_color,s.kat_color)
        };

        Self {
            time, 
            end_time,
            speed: 0.0,
            hits_required,

            hit_count: 0,
            last_hit: HitType::Don,
            complete: false,
            pos: Vector2::zero(),

            alpha_mult: 1.0,
            settings,
            
            spinner_image,
            don_color,
            kat_color
        }
    }
}
impl HitObject for TaikoSpinner {
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}
    fn note_type(&self) -> NoteType {NoteType::Spinner}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {
        // if the spinner is done, end right away
        if self.complete {self.time} else {self.end_time}
    }

    fn update(&mut self, beatmap_time: f32) {
        self.pos = self.settings.hit_position + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, 0.0);
        if beatmap_time > self.end_time {self.complete = true}
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        // if done, dont draw anything
        if self.complete {return}

        let spinner_position = Vector2::new(self.settings.hit_position.x + 100.0, self.settings.hit_position.y + 0.0);

        // if its time to start hitting the spinner
        if self.pos.x <= self.settings.hit_position.x {
            // bg circle
            list.push(Box::new(Circle::new(
                Color::YELLOW,
                -10.0,
                spinner_position,
                SPINNER_RADIUS,
                Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
            )));

            // draw another circle on top which increases in radius as the counter gets closer to the reqired
            list.push(Box::new(Circle::new(
                Color::WHITE,
                -11.0,
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
                    self.time as f64,
                    self.settings.note_radius,
                    true
                )));

                list.push(Box::new(HalfCircle::new(
                    self.kat_color,
                    self.pos,
                    self.time as f64,
                    self.settings.note_radius,
                    false
                )));
            }
        }
    }

    fn reset(&mut self) {
        self.pos.x = 0.0;
        self.hit_count = 0;
        self.complete = false;
    }
}
impl TaikoHitObject for TaikoSpinner {
    fn force_hit(&mut self) {self.complete = true}
    fn was_hit(&self) -> bool {self.complete}
    fn get_sv(&self) -> f32 {self.speed}
    fn set_sv(&mut self, sv:f32) {self.speed = sv}
    fn is_kat(&self) -> bool {self.last_hit == HitType::Don}
    fn hits_to_complete(&self) -> u32 {self.hits_required as u32}

    fn causes_miss(&self) -> bool {!self.complete} // if the spinner wasnt completed in time, cause a miss
    fn x_at(&self, time:f32) -> f32 {(self.time - time) * self.speed}
    
    fn get_points(&mut self, hit_type:HitType, time:f32, _:(f32,f32,f32)) -> ScoreHit {
        // too soon or too late
        if time < self.time || time > self.end_time {return ScoreHit::None}
        // wrong note, or already done (just in case)
        if self.last_hit == hit_type || self.complete {return ScoreHit::None}

        self.last_hit = hit_type;
        self.hit_count += 1;
        
        if self.hit_count == self.hits_required {self.complete = true}

        ScoreHit::Other(100, self.complete)
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitType {
    Don,
    Kat
}
impl Into<HitType> for KeyPress {
    fn into(self) -> HitType {
        match self {
            KeyPress::LeftKat|KeyPress::RightKat => HitType::Kat,
            KeyPress::LeftDon|KeyPress::RightDon => HitType::Don,
            _ => {panic!("mania key while playing taiko")}
        }
    }
}



#[derive(Clone)]
struct HitCircleImageHelper {
    circle: Image,
    overlay: Image,
}
impl HitCircleImageHelper {
    fn new(settings: &Arc<TaikoSettings>, depth: f64, hit_type: HitType, finisher: bool) -> Option<Self> {

        let color = match hit_type {
            HitType::Don => settings.don_color,
            HitType::Kat => settings.kat_color,
        };

        let scale;
        let hitcircle = if finisher {
            scale = settings.big_note_multiplier;
            "taikobigcircle"
        } else {
            scale = 1.0;
            "taikohitcircle"
        };


        let mut circle = SKIN_MANAGER.write().get_texture(hitcircle, true);
        if let Some(circle) = &mut circle {
            circle.depth = depth;
            circle.initial_pos = Vector2::zero();
            circle.initial_scale = Vector2::one() * scale;
            circle.initial_color = color;
            
            circle.current_pos = circle.initial_pos;
            circle.current_scale = circle.initial_scale;
            circle.current_color = circle.initial_color;
        }

        let mut overlay = SKIN_MANAGER.write().get_texture(hitcircle.to_owned() + "overlay", true);
        if let Some(overlay) = &mut overlay {
            overlay.depth = depth - 0.0000001;
            overlay.initial_pos = Vector2::zero();
            overlay.initial_scale = Vector2::one() * scale;
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

    fn set_alpha(&mut self, alpha: f32) {
        self.circle.current_color.a = alpha;
        self.overlay.current_color.a = alpha;
    }

    fn set_pos(&mut self, pos: Vector2) {
        self.circle.current_pos  = pos;
        self.overlay.current_pos = pos;
    }
    fn draw(&mut self, list: &mut Vec<Box<dyn Renderable>>) {
        list.push(Box::new(self.circle.clone()));
        list.push(Box::new(self.overlay.clone()));
    }
}
