use crate::prelude::*;

#[derive(Debug)]
pub enum ModAction {
    /// Add a mod
    AddMod(String),

    /// Remove a mod
    RemoveMod(String),

    /// Toggle a mod
    ToggleMod(String),

    /// Set the speed
    SetSpeed(f32),

    /// Add/remove to the speed
    AddSpeed(f32),

    /// Set all mods that are active
    SetMods(HashSet<String>)
}
impl From<ModAction> for TatakuAction {
    fn from(value: ModAction) -> Self {
        Self::Mods(value)
    }
}
