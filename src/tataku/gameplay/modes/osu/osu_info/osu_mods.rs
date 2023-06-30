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

pub struct Relax;
impl GameplayMod for Relax {
    fn name(&self) -> &'static str { "relax" }
    fn short_name(&self) -> &'static str { "RX" }
    fn display_name(&self) -> &'static str { "Relax" }
    fn description(&self) -> &'static str { "You just need to aim!" }

    fn score_multiplier(&self) -> f32 { 0.0 }
    fn removes(&self) -> &'static [&'static str] { &["autoplay"] }
}

/// helper for easing mods
pub struct EasingMod {
    pub(super) name: &'static str,
    pub(super) short_name: &'static str,
    pub(super) display_name: &'static str,
    pub(super) desc: &'static str,
    pub(super) removes: &'static [&'static str],
}
impl GameplayMod for EasingMod {
    fn name(&self) -> &'static str { self.name }
    fn short_name(&self) -> &'static str { self.short_name }
    fn display_name(&self) -> &'static str { self.display_name }
    fn description(&self) -> &'static str { self.desc }

    fn score_multiplier(&self) -> f32 { 1.0 }
    fn removes(&self) -> &'static [&'static str] { self.removes }
}

pub struct OnTheBeat;
impl GameplayMod for OnTheBeat {
    fn name(&self) -> &'static str { "on_the_beat" }
    fn short_name(&self) -> &'static str { "OB" }
    fn display_name(&self) -> &'static str { "On the Beat" }
    fn description(&self) -> &'static str { "Notes on beats have something off about them" }

    fn score_multiplier(&self) -> f32 { 1.0 }
    fn removes(&self) -> &'static [&'static str] { &["sine", "quad", "cube", "quart", "quint", "exp", "circ", "back", "in", "out", "inout"] }
}