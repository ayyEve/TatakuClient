#![allow(non_upper_case_globals)]
use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameplayMod {
    /// mod identifier, used in the mods hashmap
    pub name: &'static str,

    /// short (usually 2 letter) name for the mod (ie HR, EZ)
    pub short_name: &'static str,

    /// actual display name for the mod
    pub display_name: &'static str,

    /// a short description of the mod
    pub description: &'static str,

    /// texture name for this mod
    ///
    /// if this is empty when loading a texture, the loader will use the name property
    pub texture_name: &'static str,


    /// does this mod adjust the difficulty rating? used for diff calc
    pub adjusts_difficulty: bool,

    /// how much does this mod adjust the score multiplier?
    pub score_multiplier: f32,

    /// which mods is this mod incompatible with?
    pub removes: &'static [&'static str]
}
impl GameplayMod {
    pub const DEFAULT:Self = Self {
        name: "none",
        short_name: "NOPE",
        display_name: "None",
        description: "",
        texture_name: "",
        adjusts_difficulty: false,
        score_multiplier: 1.0,
        removes: &[]
    };
}
impl Default for GameplayMod {
    fn default() -> Self { Self::DEFAULT }
}
impl PartialEq for GameplayMod {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}
impl Eq for GameplayMod {}
impl AsRef<str> for GameplayMod {
    fn as_ref(&self) -> &str { self.name }
}

impl std::fmt::Display for GameplayMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

impl From<GameplayMod> for ModDefinition {
    fn from(val: GameplayMod) -> Self {
        ModDefinition {
            name: val.name.to_owned(),
            short_name: val.short_name.to_owned(),
            display_name: val.display_name.to_owned(),
            adjusts_difficulty: val.adjusts_difficulty,
            score_multiplier: val.score_multiplier,
        }
    }
}


// default mods
pub const Autoplay: GameplayMod = GameplayMod {
    name: "autoplay",
    short_name: "AT",
    display_name: "Autoplay",

    description: "Let the game play for you!",
    texture_name: "autoplay",
    
    score_multiplier: 0.0,
    adjusts_difficulty: false,
    removes: &[],
};

pub const NoFail: GameplayMod = GameplayMod {
    name: "no_fail",
    short_name: "NF",
    display_name: "No Fail",

    description: "Even if you fail, you don't!",
    texture_name: "no_fail",

    adjusts_difficulty: false,
    score_multiplier: 0.8,
    removes: &[
        "sudden_death",
        "perfect"
    ]
};

pub const SuddenDeath: GameplayMod = GameplayMod {
    name: "sudden_death",
    short_name: "SD",
    display_name: "Sudden Death",

    description: "Insta-fail if you miss",
    texture_name: "sudden_death",

    score_multiplier: 1.0,
    adjusts_difficulty: false,

    removes: &[
        "no_fail",
        "perfect"
    ]
};

pub const Perfect: GameplayMod = GameplayMod {
    name: "perfect",
    short_name: "PF",
    display_name: "Perfect",

    description: "Insta-fail if you do any less than perfect",
    texture_name: "perfect",

    score_multiplier: 1.0,
    adjusts_difficulty: false,

    removes: &[
        "no_fail",
        "sudden_death"
    ]
};
