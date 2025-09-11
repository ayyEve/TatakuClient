use crate::prelude::*;
use tataku::TatakuValue;
use common::reflect::Reflect;

/// An action that deals with the Song
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableSongAction {
    /// Play/resume the song
    Play,

    /// Pause the song
    Pause,

    /// Toggle the song (pause if playing, play if paused)
    Toggle,

    /// Restart the song from the beginning
    Restart,

    PushQueue,
    PopQueue(#[serde(default)] Box<BuildableSongPlayData>),

    /// Seek by the specified number of ms
    Seek {
        #[serde(rename="$value")]
        value: BuildableValue
    },

    /// Set the song's position
    #[serde(alias="position")]
    SetPosition {
        #[serde(rename="$value")]
        value: BuildableValue
    },

    /// Set the song's speed
    #[serde(alias="rate")]
    SetRate {
        #[serde(rename="$value")]
        value: BuildableValue
    },
}
impl BuildableSongAction {
    pub fn resolve(
        &self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>,
    ) -> Option<actions::song::SongAction> {
        match self {
            Self::Play => Some(actions::song::SongAction::Play),
            Self::Pause => Some(actions::song::SongAction::Pause),
            Self::Toggle => Some(actions::song::SongAction::Toggle),
            Self::Restart => Some(actions::song::SongAction::Restart),
            Self::PushQueue => Some(actions::song::SongAction::Set(actions::song::SongSetAction::PushQueue)),
            Self::PopQueue(value)
                => Some(actions::song::SongAction::Set(actions::song::SongSetAction::PopQueue(
                    value.resolve(values, passed_in)?,
                ))),

            Self::Seek { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(actions::song::SongAction::SeekBy),

            Self::SetPosition { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(actions::song::SongAction::SetPosition),

            Self::SetRate { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(actions::song::SongAction::SetRate),
        }
    }


    pub fn build(&mut self) {
        match self {
            Self::PopQueue(a) => a.build(),
            Self::Seek { value } => value.build(),
            Self::SetPosition { value } => value.build(),
            Self::SetRate { value } => value.build(),
            _ => {},
        }
    }
}


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct BuildableSongPlayData {
    #[serde(alias="@play", default)] pub play: BuildableValue,
    #[serde(alias="@restart", default)] pub restart: BuildableValue,
    #[serde(alias="@position", default)] pub position: BuildableValue,
    #[serde(alias="@volume", default)] pub rate: BuildableValue,
    #[serde(alias="@rate", default)] pub volume: BuildableValue,
}
impl BuildableSongPlayData {
    pub(crate) fn build(&mut self) {
        self.play.build();
        self.restart.build();
        self.position.build();
        self.rate.build();
        self.volume.build();
    }

    pub fn resolve(
        &self,
        values: &mut dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<actions::song::SongPlayData> {
        Some(actions::song::SongPlayData { 
            play: self.play
                .resolve(values, passed_in)
                .map(|i| i.as_bool())
                .unwrap_or_default(),

            restart: self.restart
                .resolve(values, passed_in)
                .map(|i| i.as_bool())
                .unwrap_or_default(),

            position: self.position
                .resolve(values, passed_in)
                .and_then(|i| i.as_f32()),

            rate: self.rate
                .resolve(values, passed_in)
                .and_then(|i| i.as_f32()),

            volume: self.volume
                .resolve(values, passed_in)
                .and_then(|i| i.as_f32()),
        })
    }
}
