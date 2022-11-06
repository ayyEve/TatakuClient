use crate::prelude::*;

pub struct FullAlt;
impl GameplayMod for FullAlt {
    fn name(&self) -> &'static str { "full_alt" }
    fn description(&self) -> &'static str { "Force full-alt :D" }
}