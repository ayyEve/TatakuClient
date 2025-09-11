use crate::*;
use common::reflect::*;
use gameplay::mods::GameplayMod;

#[repr(C)]
#[derive(Reflect)]
#[derive(Copy, Clone, Debug)]
pub struct GameplayModGroupStatic {
    pub name: &'static str,
    pub mods: &'static [ GameplayMod ],
}

#[derive(Reflect)]
#[derive(Clone, Debug)]
pub struct GameplayModGroup {
    pub name: String,
    pub mods: Vec<GameplayMod>
}
impl GameplayModGroup {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            mods: Vec::new()
        }
    }
    
    pub fn with_mod(mut self, m: GameplayMod) -> Self {
        self.mods.push(m);
        self
    }

    pub fn from_static(group: &GameplayModGroupStatic) -> Self {
        Self {
            name: group.name.to_string(),
            mods: group.mods.to_vec()
        }
    }
}

pub fn default_mod_groups() -> Vec<GameplayModGroup> {
    use gameplay::mods;
    vec![
        GameplayModGroup::new("Difficulty")
            .with_mod(mods::NoFail)
            .with_mod(mods::SuddenDeath)
            .with_mod(mods::Perfect)
        ,
        
        GameplayModGroup::new("Fun")
            .with_mod(mods::Autoplay)
        ,
    ]
}
