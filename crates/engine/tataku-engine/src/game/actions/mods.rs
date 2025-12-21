use tataku_common::GameSpeed;

use crate::*;

#[derive(Clone, Debug)]
pub enum ModAction {
    /// Push the current mods to a queue
    PushMods,

    /// Pop the latest mod collection
    PopMods,

    /// Add a mod
    AddMod(String),

    /// Remove a mod
    RemoveMod(String),

    /// Toggle a mod
    ToggleMod(String),

    /// Set the speed
    SetSpeed(GameSpeed),

    /// Add/remove to the speed
    AddSpeed(GameSpeed),

    /// Set all mods that are active
    SetMods(HashSet<String>)
}
impl From<ModAction> for actions::Action {
    fn from(value: ModAction) -> Self {
        Self::Mods(value)
    }
}
