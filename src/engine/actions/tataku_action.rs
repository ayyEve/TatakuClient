use crate::prelude::*;

/// An action from a menu (or dialog) to tell the game to do something
pub enum TatakuAction {
    /// Don't do anything (this is a helper)
    None,

    /// Perform a menu operation
    Menu(MenuMenuAction),

    /// Perform a game operation
    Game(GameAction),

    /// Perform a beatmap operation
    Beatmap(BeatmapAction),

    /// Perform an operation on the current song
    Song(SongAction),

    /// Perform a widget operation
    PerformOperation(IcedOperation),

    /// Handle a multiplayer action
    Multiplayer(MultiplayerAction),

    /// Quit the game
    Quit
}

// impl TatakuAction {
//     pub fn set_menu(menu: impl AsyncMenu + 'static) -> Self {
//         Self::Menu(MenuMenuAction::SetMenu(Box::new(menu)))
//     }
// }

impl std::fmt::Debug for TatakuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Menu(menu) => write!(f, "Menu({menu:?})"),
            Self::Game(game) => write!(f, "Game(TODO)"),
            Self::Beatmap(map) => write!(f, "Beatmap({map:?})"),
            Self::Song(song) => write!(f, "Song({song:?})"),
            Self::PerformOperation(_) => write!(f, "PerformOperation"),
            Self::Multiplayer(multi) => write!(f, "Multiplayer({multi:?})"),
            Self::Quit => write!(f, "Quit"),
        }
    }
}