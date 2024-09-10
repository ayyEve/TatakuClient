use crate::prelude::*;

#[derive(Debug)]
pub enum ModAction {
    /// add a mod
    AddMod(String),

    /// remove a mod
    RemoveMod(String),

    /// toggle a mod
    ToggleMod(String),


    /// set the speed
    SetSpeed(f32),

    /// add/remove to the speed
    AddSpeed(f32)
}
impl From<ModAction> for TatakuAction {
    fn from(value: ModAction) -> Self {
        Self::Mods(value)
    }
}