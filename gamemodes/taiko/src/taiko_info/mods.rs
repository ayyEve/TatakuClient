#![allow(non_upper_case_globals)]
use crate::prelude::*;

pub const FullAlt: GameplayMod = GameplayMod {
    name: "full_alt",
    short_name: "FA",
    display_name: "Full Alt",
    description: "Force full-alt :D",
    ..GameplayMod::DEFAULT
};

pub const NoSV: GameplayMod = GameplayMod {
    name: "no_sv",
    short_name: "NS",
    display_name: "No SV",
    description: "No more slider velocity changes!",
    ..GameplayMod::DEFAULT
};

pub const Relax: GameplayMod = GameplayMod {
    name: "relax",
    short_name: "RX",
    display_name: "Relax",
    description: "Hit any (taiko) key you want!",
    ..GameplayMod::DEFAULT
};

pub const HardRock:GameplayMod = GameplayMod {
    name: "hardrock",
    short_name: "HR",
    display_name: "Hard Rock",
    description: "Timing is tigher >:3",
    score_multiplier: 1.4,

    ..GameplayMod::DEFAULT
};

pub const Easy: GameplayMod = GameplayMod {
    name: "easy",
    short_name: "EZ",
    display_name: "Easy",
    description: "Timing is looser :3",
    score_multiplier: 0.6,
    
    ..GameplayMod::DEFAULT
};

pub const NoBattery: GameplayMod = GameplayMod {
    name: "no_battery",
    short_name: "NB",
    display_name: "No Battery",
    description: "Don't use battery health",
    
    ..GameplayMod::DEFAULT
};

pub const NoFinisher: GameplayMod = GameplayMod {
    name: "no_finisher",
    short_name: "NX",
    display_name: "No Finishers",
    description: "Turn all big notes into small notes",
    ..GameplayMod::DEFAULT
};


pub const Flashlight: GameplayMod = GameplayMod {
    name: "flashlight",
    short_name: "FL",
    display_name: "Flashlight",
    description: "Waaa I can't see anything!",
    score_multiplier: 1.5,

    ..GameplayMod::DEFAULT
};