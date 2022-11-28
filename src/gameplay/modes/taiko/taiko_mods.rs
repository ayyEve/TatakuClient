use crate::prelude::*;

pub struct FullAlt;
impl GameplayMod for FullAlt {
    fn name(&self) -> &'static str { "full_alt" }
    fn description(&self) -> &'static str { "Force full-alt :D" }
}

pub struct NoSV;
impl GameplayMod for NoSV {
    fn name(&self) -> &'static str { "no_sv" }
    fn description(&self) -> &'static str { "No more slider velocity changes!" }
}

pub struct HardRock;
impl GameplayMod for HardRock {
    fn name(&self) -> &'static str { "hardrock" }
    fn description(&self) -> &'static str { "Timing is tigher >:3" }
}

pub struct Easy;
impl GameplayMod for Easy {
    fn name(&self) -> &'static str { "easy" }
    fn description(&self) -> &'static str { "Timing is looser :3" }
}