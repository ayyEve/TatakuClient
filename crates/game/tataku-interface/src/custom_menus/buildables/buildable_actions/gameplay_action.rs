use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableGameplayAction {
    /// Quit a pending game
    Quit,

    /// Resume a pending game
    Resume,

    /// Pause a current game
    Pause,

    /// Retry a pending game
    Retry,
}
#[cfg(feature="graphics")]
impl BuildableGameplayAction {
    pub fn resolve(&self) -> actions::game::CurrentGameAction {
        match self {
            Self::Pause => actions::game::CurrentGameAction::Pause {
                id: "pause_menu".to_owned(),
            },
            Self::Quit => actions::game::CurrentGameAction::Free,
            Self::Resume => actions::game::CurrentGameAction::Resume,
            Self::Retry => actions::game::CurrentGameAction::Restart,
        }
    }
}
