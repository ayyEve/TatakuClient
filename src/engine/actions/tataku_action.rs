use crate::prelude::*;

/// An action from a menu (or dialog) to tell the game to do something
#[derive(Default)]
pub enum TatakuAction {
    /// Don't do anything (this is a helper)
    #[default]
    None,

    /// Perform a menu operation
    Menu(MenuAction),

    /// Perform a game operation
    Game(GameAction),

    /// Perform a beatmap operation
    Beatmap(BeatmapAction),

    /// Perform an operation on the current song
    Song(SongAction),

    /// Perform a widget operation
    PerformOperation(IcedOperation),

    /// Perform a multiplayer action
    Multiplayer(MultiplayerAction),

    /// Perform a task action
    Task(TaskAction),

    /// Perform a cursor action
    CursorAction(CursorAction),
}
impl std::fmt::Debug for TatakuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Menu(menu) => write!(f, "Menu({menu:?})"),
            Self::Game(game) => write!(f, "Game({game:?})"),
            Self::Beatmap(map) => write!(f, "Beatmap({map:?})"),
            Self::Song(song) => write!(f, "Song({song:?})"),
            Self::PerformOperation(_) => write!(f, "PerformOperation"),
            Self::Multiplayer(multi) => write!(f, "Multiplayer({multi:?})"),
            Self::Task(task) => write!(f, "Task({task:?})"),
            Self::CursorAction(action) => write!(f, "CursorAction({action:?})"),
        }
    }
}
