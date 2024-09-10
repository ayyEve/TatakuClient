#![allow(non_upper_case_globals)]
use crate::prelude::*;

pub const TaikoStatLeftPresses: GameModeStat = GameModeStat {
    name: "count_left",
    display_name: "Left Presses",
    description: ""
};

pub const TaikoStatRightPresses: GameModeStat = GameModeStat {
    name: "count_right",
    display_name: "Right Presses",
    description: ""
};

pub const TaikoPressCounterStatGroup: StatGroup = StatGroup {
    name: "press_counters", 
    display_name: "Press Counts",
    stats: & [
        TaikoStatLeftPresses,
        TaikoStatRightPresses,
    ]
};