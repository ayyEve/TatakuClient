use core::f32;

use piston::RenderArgs;

use taiko_rs_common::types::KeyPress;
use taiko_rs_common::types::ScoreHit;
use crate::Vector2;
use crate::gameplay::BAR_COLOR;
use crate::gameplay::HitObject;
use crate::gameplay::NoteType;
use super::{HIT_POSITION, NOTE_RADIUS};
use crate::render::{Circle, Color, HalfCircle, Rectangle, Renderable, Border};

const SLIDER_DOT_RADIUS:f64 = 8.0;
const SPINNER_RADIUS:f64 = 200.0;
const SPINNER_POSITION:Vector2 = Vector2::new(HIT_POSITION.x + 100.0, HIT_POSITION.y + 0.0);
const FINISHER_LENIENCY:f32 = 20.0; // ms
const NOTE_BORDER_SIZE:f64 = 2.0;

const GRAVITY_SCALING:f32 = 400.0;

const DON_COLOR:Color = Color::new(1.0, 0.0, 0.0, 1.0);
const KAT_COLOR:Color = Color::new(0.0, 0.0, 1.0, 1.0);


pub trait TaikoHitObject: HitObject {
    fn is_kat(&self) -> bool {false}// needed for diff calc :/

    fn get_sv(&self) -> f32;
    fn set_sv(&mut self, sv:f32);
    /// does this hit object play a finisher sound when hit?
    fn finisher_sound(&self) -> bool {false}

    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool; //TODO: might change this to return an enum of "no", "yes". "yes_combo_only"
    
    fn get_points(&mut self, hit_type:HitType, time:f32, hit_windows:(f32,f32,f32)) -> ScoreHit; // if negative, counts as a miss
    fn check_finisher(&mut self, _hit_type:HitType, _time:f32) -> ScoreHit {ScoreHit::None}


    fn x_at(&self, time:f32) -> f32 {(self.time() - time) * self.get_sv()}
}


// note
#[derive(Clone, Copy)]
pub struct TaikoNote {
    pos: Vector2,
    time: f32, // ms
    hit_time: f32,
    hit_type: HitType,
    finisher: bool,
    hit: bool,
    missed: bool,
    speed: f32
}
impl TaikoNote {
    pub fn new(time:f32, hit_type:HitType, finisher:bool, speed:f32) -> Self {
        Self {
            time, 
            hit_time: 0.0,
            hit_type, 
            finisher,
            speed,
            hit: false,
            missed: false,
            pos: Vector2::zero(),
        }
    }

    fn get_color(&mut self) -> Color {
        match self.hit_type {
            HitType::Don => DON_COLOR,
            HitType::Kat => KAT_COLOR,
        }
    }
}
impl HitObject for TaikoNote {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, hw_miss:f32) -> f32 {self.time + hw_miss}
    fn update(&mut self, beatmap_time: f32) {
        let y = 
            if self.hit {-((beatmap_time - self.hit_time)*20.0).ln()*20.0 + 1.0} 
            else if self.missed {GRAVITY_SCALING * 9.81 * ((beatmap_time - self.hit_time)/1000.0).powi(2)} 
            else {0.0};
        
        self.pos = HIT_POSITION + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, y as f64);
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.x + NOTE_RADIUS < 0.0 || self.pos.x - NOTE_RADIUS > args.window_size[0] as f64 {return}

        let mut note = Circle::new(
            self.get_color(),
            self.time as f64,
            self.pos,
            if self.finisher {NOTE_RADIUS*1.6666} else {NOTE_RADIUS}
        );
        note.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
        list.push(Box::new(note));
    }

    fn reset(&mut self) {
        self.pos = Vector2::zero();
        self.hit = false;
        self.missed = false;
        self.hit_time = 0.0;
    }
}
impl TaikoHitObject for TaikoNote {
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
    end_x: f64
}
impl TaikoSlider {
    pub fn new(time:f32, end_time:f32, finisher:bool, speed:f32) -> Self {
        let radius = if finisher {NOTE_RADIUS*1.6666} else {NOTE_RADIUS};

        Self {
            time, 
            end_time,
            finisher,
            speed,
            radius,

            pos: Vector2::new(0.0,HIT_POSITION.y - radius),
            end_x: 0.0,
            hit_dots: Vec::new()
        }
    }
}
impl HitObject for TaikoSlider {
    fn note_type(&self) -> NoteType {NoteType::Slider}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {self.end_time}
    fn update(&mut self, beatmap_time: f32) {
        self.pos.x = HIT_POSITION.x + ((self.time - beatmap_time) * self.speed) as f64;
        self.end_x = HIT_POSITION.x + ((self.end_time(0.0) - beatmap_time) * self.speed) as f64;

        // draw hit dots
        for dot in self.hit_dots.as_mut_slice() {
            if dot.done {continue;}
            dot.update(beatmap_time);
        }
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.end_x + NOTE_RADIUS < 0.0 || self.pos.x - NOTE_RADIUS > args.window_size[0] as f64 {return}

        // middle
        list.push(Box::new(Rectangle::new(
            Color::YELLOW,
            self.time as f64 + 1.0,
            self.pos,
            Vector2::new(self.end_x - self.pos.x , self.radius * 2.0),
            Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE))
        )));

        // start circle
        let mut start_c = Circle::new(
            Color::YELLOW,
            self.time as f64,
            self.pos + Vector2::new(0.0, self.radius),
            self.radius
        );
        start_c.border = Some(Border {
            color: Color::BLACK.into(),
            radius: NOTE_BORDER_SIZE
        });
        list.push(Box::new(start_c));
        
        // end circle
        let mut end_c = Circle::new(
            Color::YELLOW,
            self.time as f64,
            Vector2::new(self.end_x, self.pos.y + self.radius),
            self.radius
        );
        end_c.border = Some(Border {
            color: Color::BLACK.into(),
            radius: NOTE_BORDER_SIZE
        });
        list.push(Box::new(end_c));

        // draw hit dots
        for dot in self.hit_dots.as_slice() {
            if dot.done {continue}
            list.extend(dot.draw());
        }
    }

    fn reset(&mut self) {
        self.hit_dots.clear();
        self.pos.x = 0.0;
        self.end_x = 0.0;
    }
}
impl TaikoHitObject for TaikoSlider {
    fn get_sv(&self) -> f32 {self.speed}
    fn set_sv(&mut self, sv:f32) {self.speed = sv}
    fn causes_miss(&self) -> bool {false}

    fn get_points(&mut self, _hit_type:HitType, time:f32, _:(f32,f32,f32)) -> ScoreHit {
        // too soon or too late
        if time < self.time || time > self.end_time {return ScoreHit::None}

        self.hit_dots.push(SliderDot::new(time, self.speed));
        ScoreHit::Other(100, false)
    }

}
/// helper struct for drawing hit slider points
#[derive(Clone, Copy)]
struct SliderDot {
    time: f32,
    speed: f32,
    pos: Vector2,
    pub done: bool
}
impl SliderDot {
    pub fn new(time:f32, speed:f32) -> SliderDot {
        SliderDot {
            time,
            speed,
            pos: Vector2::zero(),
            done: false
        }
    }
    pub fn update(&mut self, beatmap_time:f32) {
        let y = -((beatmap_time - self.time)*20.0).ln()*20.0 + 1.0;
        self.pos = HIT_POSITION + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, y as f64);
        
        if !self.done && self.pos.x - SLIDER_DOT_RADIUS <= 0.0 {
            self.done = true;
        }
    }
    pub fn draw(&self) -> [Box<dyn Renderable>; 2] {

        let mut c = Circle::new(
            Color::YELLOW,
            -100.0,
            self.pos,
            SLIDER_DOT_RADIUS
        );
        c.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE/2.0));

        [
            Box::new(c),
            // "hole punch"
            Box::new(Circle::new(
                BAR_COLOR,
                0.0,
                Vector2::new(self.pos.x, HIT_POSITION.y),
                SLIDER_DOT_RADIUS
            )),
        ]
    }
}

// spinner
#[derive(Clone, Copy)]
pub struct TaikoSpinner {
    pos: Vector2, // the note in the bar, not the spinner itself
    hit_count: u16,
    last_hit: HitType,
    complete: bool, // is this spinner done

    hits_required: u16, // how many hits until the spinner is "done"
    time: f32, // ms
    end_time: f32, // ms
    speed: f32,
}
impl TaikoSpinner {
    pub fn new(time:f32, end_time:f32, speed:f32, hits_required:u16) -> Self {
        Self {
            time, 
            end_time,
            speed,
            hits_required,

            hit_count: 0,
            last_hit: HitType::Don,
            complete: false,
            pos: Vector2::zero()
        }
    }
}
impl HitObject for TaikoSpinner {
    fn note_type(&self) -> NoteType {NoteType::Spinner}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self,_:f32) -> f32 {
        // if the spinner is done, end right away
        if self.complete {self.time} else {self.end_time}
    }

    fn update(&mut self, beatmap_time: f32) {
        self.pos = HIT_POSITION + Vector2::new(((self.time - beatmap_time) * self.speed) as f64, 0.0);
        if beatmap_time > self.end_time {self.complete = true}
    }
    fn draw(&mut self, _args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        // if done, dont draw anything
        if self.complete {return}

        // if its time to start hitting the spinner
        if self.pos.x <= HIT_POSITION.x {
            // bg circle
            let mut bg = Circle::new(
                Color::YELLOW,
                -10.0,
                SPINNER_POSITION,
                SPINNER_RADIUS
            );
            bg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
            list.push(Box::new(bg));

            // draw another circle on top which increases in radius as the counter gets closer to the reqired
            let mut fg = Circle::new(
                Color::WHITE,
                -11.0,
                SPINNER_POSITION,
                SPINNER_RADIUS * (self.hit_count as f64 / self.hits_required as f64)
            );
            fg.border = Some(Border::new(Color::BLACK, NOTE_BORDER_SIZE));
            list.push(Box::new(fg));
            
            //TODO: draw a counter

        } else { // just draw the note on the playfield
            let h1 = HalfCircle::new(
                Color::BLUE,
                self.pos,
                self.time as f64,
                NOTE_RADIUS,
                false
            );
            list.push(Box::new(h1));

            let h2 = HalfCircle::new(
                Color::RED,
                self.pos,
                self.time as f64,
                NOTE_RADIUS,
                true
            );
            list.push(Box::new(h2));
        }
    }

    fn reset(&mut self) {
        self.pos.x = 0.0;
        self.hit_count = 0;
        self.complete = false;
    }
}
impl TaikoHitObject for TaikoSpinner {
    fn get_sv(&self) -> f32 {self.speed}
    fn set_sv(&mut self, sv:f32) {self.speed = sv}

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

