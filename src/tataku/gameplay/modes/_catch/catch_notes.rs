use piston::RenderArgs;

use super::catch::hit_y;
use crate::prelude::*;

const NOTE_BORDER_SIZE:f64 = 2.0;


pub trait CatchHitObject: HitObject {
    /// does this object count as a miss if it is not hit?
    fn causes_miss(&self) -> bool;
    fn get_points(&mut self) -> ScoreHit;
    fn speed(&self) -> f32;
    fn radius(&self) -> f64;
    fn x(&self) -> f64;
    fn y_at(&self, time:f32) -> f64 {
        hit_y() 
        - (
            (self.time() - time) 
            * self.speed() 
            - (self.radius() + NOTE_BORDER_SIZE / 2.0) as f32
        ) as f64
    }

    fn set_dash(&mut self, _next: &Box<dyn CatchHitObject>) {}
    fn reset_dash(&mut self) {}

    fn was_hit(&self) -> bool;
    fn pos_at(&self, time:f32, scaling_helper: &ScalingHelper) -> f64;
}


// normal note
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct CatchFruit {
    pos: Vector2,
    time: f32, // ms
    hit: bool,
    missed: bool,
    speed: f32,
    radius: f64,

    dash: bool,
    dash_distance: f32,

    alpha_mult: f32,
}
impl CatchFruit {
    pub fn new(time:f32, speed:f32, radius:f64, x:f64) -> Self {
        Self {
            time, 
            speed,
            radius,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),

            dash: true,
            dash_distance: 0.0,
            alpha_mult: 1.0
        }
    }
}
impl HitObject for CatchFruit {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, _:f32) -> f32 {self.time}
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}

    fn update(&mut self, beatmap_time: f32) {
        self.pos.y = self.y_at(beatmap_time);
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.y + self.radius < 0.0 || self.pos.y - self.radius > args.window_size[1] as f64 || self.hit {return}

        list.push(Box::new(Circle::new(
            Color::BLUE.alpha(self.alpha_mult),
            -100.0,
            self.pos,
            self.radius,
            Some(Border::new((if self.dash {Color::RED} else {Color::BLACK}).alpha(self.alpha_mult), NOTE_BORDER_SIZE))
        )));
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
        self.missed = false;
    }
}
impl CatchHitObject for CatchFruit {
    fn was_hit(&self) -> bool {self.hit||self.missed}
    fn pos_at(&self, _time:f32, _scaling_helper: &ScalingHelper) -> f64 {self.pos.x}
    fn x(&self) -> f64 {self.pos.x}
    fn speed(&self) -> f32 {self.speed}
    fn radius(&self) -> f64 {self.radius}
    fn causes_miss(&self) -> bool {true}
    fn get_points(&mut self) -> ScoreHit {
        self.hit = true;
        ScoreHit::X300
    }

    fn set_dash(&mut self, next: &Box<dyn CatchHitObject>) {
        let _distance_to = (self.pos.x - next.x()).abs();

        // if distance_to
    }
}


// slider droplet
#[derive(Clone, Copy)]
pub struct CatchDroplet {
    pos: Vector2,
    time: f32, // ms
    hit: bool,
    missed: bool,
    speed: f32,
    radius: f64,

    alpha_mult: f32
}
impl CatchDroplet {
    pub fn new(time:f32, speed:f32, radius:f64, x:f64) -> Self {
        Self {
            time, 
            speed,
            radius,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),
            alpha_mult: 1.0
        }
    }
}
impl HitObject for CatchDroplet {
    fn note_type(&self) -> NoteType {NoteType::Note}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, _:f32) -> f32 {self.time}
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}
    fn update(&mut self, beatmap_time: f32) {
        self.pos.y = self.y_at(beatmap_time);
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>){
        if self.pos.y + self.radius < 0.0 || self.pos.y - self.radius > args.window_size[1] || self.hit {return}

        list.push(Box::new(Circle::new(
            Color::BLUE.alpha(self.alpha_mult),
            -100.0,
            self.pos,
            self.radius,
            Some(Border::new(Color::BLACK.alpha(self.alpha_mult), NOTE_BORDER_SIZE))
        )));
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
        self.missed = false;
    }
}
impl CatchHitObject for CatchDroplet {
    fn was_hit(&self) -> bool {self.hit||self.missed}
    fn pos_at(&self, _time:f32, _scaling_helper: &ScalingHelper) -> f64 {self.pos.x}
    fn x(&self) -> f64 {self.pos.x}
    fn speed(&self) -> f32 {self.speed}
    fn radius(&self) -> f64 {self.radius}
    fn causes_miss(&self) -> bool {true}
    fn get_points(&mut self) -> ScoreHit {
        self.hit = true;
        ScoreHit::X100
    }
}


// spinner banana
#[derive(Clone, Copy)]
pub struct CatchBanana {
    pos: Vector2,
    time: f32, // ms
    hit: bool,
    missed: bool,
    speed: f32,
    radius: f64,

    alpha_mult: f32
}
impl CatchBanana {
    pub fn new(time:f32, speed:f32, radius:f64, x:f64) -> Self {
        Self {
            time, 
            speed,
            radius,
            hit: false,
            missed: false,
            pos: Vector2::new(x, 0.0),
            alpha_mult: 1.0
        }
    }
}
impl HitObject for CatchBanana {
    fn note_type(&self) -> NoteType {NoteType::Spinner}
    fn time(&self) -> f32 {self.time}
    fn end_time(&self, _:f32) -> f32 {self.time}
    fn set_alpha(&mut self, alpha: f32) {self.alpha_mult = alpha}
    fn update(&mut self, beatmap_time: f32) {
        self.pos.y = self.y_at(beatmap_time);
    }
    fn draw(&mut self, args:RenderArgs, list: &mut Vec<Box<dyn Renderable>>) {
        if self.pos.y + self.radius < 0.0 || self.pos.y - self.radius > args.window_size[1] as f64 || self.hit {return}

        list.push(Box::new(Circle::new(
            Color::YELLOW.alpha(self.alpha_mult),
            -100.0,
            self.pos,
            self.radius,
            Some(Border::new(Color::BLACK.alpha(self.alpha_mult), NOTE_BORDER_SIZE))
        )));
    }

    fn reset(&mut self) {
        self.pos.y = 0.0;
        self.hit = false;
    }
}
impl CatchHitObject for CatchBanana {
    fn was_hit(&self) -> bool {self.hit||self.missed}
    fn pos_at(&self, _time:f32, _scaling_helper: &ScalingHelper) -> f64 {self.pos.x}
    fn x(&self) -> f64 {self.pos.x}
    fn speed(&self) -> f32 {self.speed}
    fn radius(&self) -> f64 {self.radius}
    fn causes_miss(&self) -> bool {false}
    fn get_points(&mut self) -> ScoreHit {
        self.hit = true;
        ScoreHit::Other(10, true)
    }
}
