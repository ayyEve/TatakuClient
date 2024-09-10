use crate::prelude::*;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GameplayModGroupStatic {
    pub name: &'static str,
    pub mods: &'static [GameplayMod],
}

#[derive(Clone)]
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
    vec![
        GameplayModGroup::new("Difficulty")
            .with_mod(NoFail)
            .with_mod(SuddenDeath)
            .with_mod(Perfect)
        ,
        
        GameplayModGroup::new("Fun")
            .with_mod(Autoplay)
        ,
    ]
}
