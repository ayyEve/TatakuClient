use crate::prelude::*;

// pub struct Hidden;
// pub struct Flashlight;

pub struct Easy;
impl GameplayMod for Easy {
    fn name(&self) -> &'static str { "easy" }
    fn short_name(&self) -> &'static str { "EZ" }
    fn display_name(&self) -> &'static str { "Easy" }
    fn description(&self) -> &'static str { "bigger and slower notes c:" }
    
    fn score_multiplier(&self) -> f32 { 0.6 }
    fn removes(&self) -> &'static [&'static str] { &["hardrock"] }
}


pub struct HardRock;
impl GameplayMod for HardRock {
    fn name(&self) -> &'static str { "hardrock" }
    fn short_name(&self) -> &'static str { "HR" }
    fn display_name(&self) -> &'static str { "Hard Rock" }
    fn description(&self) -> &'static str { "smaller notes, higher approach, what fun!" }

    fn score_multiplier(&self) -> f32 { 1.4 }
    fn removes(&self) -> &'static [&'static str] { &["easy"] }
}
