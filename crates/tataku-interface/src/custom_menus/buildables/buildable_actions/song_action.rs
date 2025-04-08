use crate::prelude::*;

/// An action that deals with the Song
#[derive(Clone, Debug, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildableSongAction {
    /// Play/resume the song
    Play,

    /// Pause the song
    Pause,

    /// Toggle the song (pause if playing, play if paused)
    Toggle,

    /// Restart the song from the beginning
    Restart,

    /// Seek by the specified number of ms
    Seek { 
        #[serde(rename="$value", alias="$text", alias="@seek")] 
        value: BuildableValue
    },

    /// Set the song's position
    #[serde(alias="position")]
    SetPosition { 
        #[serde(rename="$value", alias="$text", alias="@position")] 
        value: BuildableValue
    },

    /// Set the song's speed
    #[serde(alias="rate")]
    SetRate { 
        #[serde(rename="$value", alias="$text", alias="@rate")] 
        value: BuildableValue
    },
}
impl BuildableSongAction {
    pub fn into_action(self, values: &mut dyn Reflect) -> Option<SongAction> {
        match self {
            Self::Play => Some(SongAction::Play),
            Self::Pause => Some(SongAction::Pause),
            Self::Toggle => Some(SongAction::Toggle),
            Self::Restart => Some(SongAction::Restart),
            Self::Seek { value: n } => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(SongAction::SeekBy),
            Self::SetPosition { value: n } => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(SongAction::SetPosition),
            Self::SetRate { value: n } => n.resolve(values, None).and_then(|n| n.as_f32().ok()).map(SongAction::SetRate),
        }
    }


    pub fn build(&mut self, values: &dyn Reflect) {
        let thing = match self {
            Self::Seek { value } => value,
            Self::SetPosition { value } => value,
            Self::SetRate { value } => value,
            _ => return,
        };

        thing.resolve_pre(values);
    }
}
