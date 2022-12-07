use crate::prelude::*;

pub struct TaikoStatLeftPresses;
impl GameModeStat for TaikoStatLeftPresses {
    fn name(&self) -> &'static str { "count_left" }
    fn display_name(&self) -> &'static str { "Left Presses" }
}

pub struct TaikoStatRightPresses;
impl GameModeStat for TaikoStatRightPresses {
    fn name(&self) -> &'static str { "count_right" }
    fn display_name(&self) -> &'static str { "Right Presses" }
}
