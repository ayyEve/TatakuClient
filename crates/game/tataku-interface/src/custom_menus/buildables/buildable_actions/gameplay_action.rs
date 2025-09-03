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
impl BuildableGameplayAction {
    pub fn into_action(self) -> CurrentGameAction {
        match self {
            Self::Pause => CurrentGameAction::Pause {
                id: "pause_menu".to_owned(),
            },
            Self::Quit => CurrentGameAction::Free,
            Self::Resume => CurrentGameAction::Resume,
            Self::Retry => CurrentGameAction::Restart,
        }
    }
}
