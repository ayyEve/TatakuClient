use crate::prelude::*;

pub struct FullAlt;
impl GameplayMod for FullAlt {
    fn name(&self) -> &'static str { "full_alt" }
    fn short_name(&self) -> &'static str { "FA" }
    fn display_name(&self) -> &'static str { "Full Auto" }
    fn description(&self) -> &'static str { "Force full-alt :D" }
}

pub struct NoSV;
impl GameplayMod for NoSV {
    fn name(&self) -> &'static str { "no_sv" }
    fn short_name(&self) -> &'static str { "NS" }
    fn display_name(&self) -> &'static str { "No SV" }
    fn description(&self) -> &'static str { "No more slider velocity changes!" }
}
pub struct Relax;
impl GameplayMod for Relax {
    fn name(&self) -> &'static str { "relax" }
    fn short_name(&self) -> &'static str { "RX" }
    fn display_name(&self) -> &'static str { "Relax" }
    fn description(&self) -> &'static str { "Hit any (taiko) key you want!" }
}

pub struct HardRock;
impl GameplayMod for HardRock {
    fn name(&self) -> &'static str { "hardrock" }
    fn short_name(&self) -> &'static str { "HR" }
    fn display_name(&self) -> &'static str { "Hard Rock" }
    fn description(&self) -> &'static str { "Timing is tigher >:3" }

    fn score_multiplier(&self) -> f32 { 1.4 }
}

pub struct Easy;
impl GameplayMod for Easy {
    fn name(&self) -> &'static str { "easy" }
    fn short_name(&self) -> &'static str { "EZ" }
    fn display_name(&self) -> &'static str { "Easy" }
    fn description(&self) -> &'static str { "Timing is looser :3" }

    fn score_multiplier(&self) -> f32 { 0.6 }
}