use crate::prelude::*;

// pub struct Hidden;
// pub struct Flashlight;

pub struct Easy;
impl GameplayMod for Easy {
    fn name(&self) -> &'static str { "easy" }
    fn score_multiplier(&self) -> f32 { 0.6 }
    fn description(&self) -> &'static str { "bigger and slower notes c:" }
}


pub struct HardRock;
impl GameplayMod for HardRock {
    fn name(&self) -> &'static str { "hardrock" }
    fn score_multiplier(&self) -> f32 { 1.4 }
    fn description(&self) -> &'static str { "smaller notes, higher approach, what fun!" }
}
