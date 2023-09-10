#![allow(non_upper_case_globals)]
use crate::prelude::*;

// pub struct Hidden;

pub struct Flashlight;
impl GameplayMod for Flashlight {
    fn name(&self) -> &'static str { "flashlight" }
    fn short_name(&self) -> &'static str { "FL" }
    fn display_name(&self) -> &'static str { "Flashlight" }
    fn description(&self) -> &'static str { "Waaa I can't see anything!" }
    
    fn score_multiplier(&self) -> f32 { 1.1 }
    // fn removes(&self) -> &'static [&'static str] { &["hardrock"] }
}


pub const Easy:GameplayMod = GameplayMod {
    name: "easy", 
    short_name: "EZ", 
    display_name: "Easy", 
    description: "Bigger and slower notes c:", 
    texture_name: "easy", 
    score_multiplier: 0.6, 

    adjusts_difficulty: false,
    removes: &[HardRock.name]
};

pub const HardRock:GameplayMod = GameplayMod {
    name: "hardrock", 
    short_name: "HR", 
    display_name: "Hard Rock", 
    description: "Smaller notes, higher approach, what fun!", 
    texture_name: "hardrock", 
    score_multiplier: 1.4, 

    adjusts_difficulty: false,
    removes: &["easy"]
};

pub const Relax:GameplayMod = GameplayMod {
    name: "relax", 
    short_name: "RX", 
    display_name: "Relax", 
    description: "You just need to aim!", 
    texture_name: "relax", 
    score_multiplier: 0.0, 

    adjusts_difficulty: false,
    removes: &["autoplay"]
};

pub const OnTheBeat:GameplayMod = GameplayMod {
    name: "on_the_beat", 
    short_name: "OB", 
    display_name: "On the Beat", 
    description: "Notes on beats have something off about them", 
    texture_name: "relax", 
    score_multiplier: 1.0, 

    adjusts_difficulty: false,
    removes: &[
        "sine", 
        "quad", 
        "cube", 
        "quart", 
        "quint", 
        "exp", 
        "circ", 
        "back", 
        "in", 
        "out", 
        "inout"
    ]
};