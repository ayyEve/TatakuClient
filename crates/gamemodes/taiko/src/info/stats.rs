#![allow(non_upper_case_globals)]
use crate::prelude::*;

use engine::gameplay::stats::{
    StatGroup,
    GameModeStat,
};

pub const LeftPresses: GameModeStat = GameModeStat {
    name: "count_left",
    display_name: "Left Presses",
    description: ""
};

pub const RightPresses: GameModeStat = GameModeStat {
    name: "count_right",
    display_name: "Right Presses",
    description: ""
};

pub const PressCounter: StatGroup = StatGroup {
    name: "press_counters",
    display_name: "Press Counts",
    stats: & [
        LeftPresses,
        RightPresses,
    ]
};
