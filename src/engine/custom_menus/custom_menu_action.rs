use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum CustomMenuAction {
    /// No action
    None,

    /// Set the menu
    SetMenu(String),

    /// Add a dialog
    AddDialog(String),

    /// Perform a map action
    Map(CustomMenuMapAction),

    /// Perform a song action
    Song(CustomMenuSongAction),
}
impl Into<MenuAction> for CustomMenuAction {
    fn into(self) -> MenuAction {
        match self {
            Self::None => MenuAction::None,
            Self::AddDialog(dialog) => MenuAction::Menu(MenuMenuAction::AddDialogCustom(dialog, true)),
            Self::SetMenu(menu) => MenuAction::Menu(MenuMenuAction::SetMenuCustom(menu)),

            Self::Map(action) => MenuAction::Beatmap(action.into()),
            Self::Song(action) => MenuAction::Song(action.into()),
        }
    }
}


/// An action that deals with the current beatmap
#[derive(Clone, Debug)]
pub enum CustomMenuMapAction {
    /// Play the current map
    Play,

    /// Change to the next map
    Next,

    /// Change to the previous map
    Previous(MapActionIfNone),

    /// Change to a random map
    Random(bool),
}
impl Into<BeatmapMenuAction> for CustomMenuMapAction {
    fn into(self) -> BeatmapMenuAction {
        match self {
            Self::Play => BeatmapMenuAction::PlaySelected,
            Self::Next => BeatmapMenuAction::Next,
            Self::Previous(action) => BeatmapMenuAction::Previous(action),
            Self::Random(use_preview) => BeatmapMenuAction::Random(use_preview),
        }
    }
}

/// An action that deals with the Song
#[derive(Clone, Debug)]
pub enum CustomMenuSongAction {
    /// Play/resume the song
    Play,

    /// Pause the song
    Pause,

    /// Restart the song from the beginning
    Restart,

    /// Seek by the specified number of ms
    Seek(f32),

    /// Set the song's position
    SetPosition(f32),
}
impl Into<SongMenuAction> for CustomMenuSongAction {
    fn into(self) -> SongMenuAction {
        match self {
            Self::Play => SongMenuAction::Play,
            Self::Pause => SongMenuAction::Pause,
            Self::Restart => SongMenuAction::Restart,
            Self::Seek(n) => SongMenuAction::SeekBy(n),
            Self::SetPosition(p) => SongMenuAction::SetPosition(p)
        }
    }
}