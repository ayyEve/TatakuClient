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

    PushQueue,
    PopQueue {
        #[serde(rename="$value", alias="$text", default)] 
        value: Box<BuildableSongPlayData>,
    },

    /// Seek by the specified number of ms
    Seek { 
        #[serde(rename="$value", alias="$text", alias="@seek")] 
        value: BuildableValue,
    },

    /// Set the song's position
    #[serde(alias="position")]
    SetPosition { 
        #[serde(rename="$value", alias="$text", alias="@position")] 
        value: BuildableValue,
    },

    /// Set the song's speed
    #[serde(alias="rate")]
    SetRate { 
        #[serde(rename="$value", alias="$text", alias="@rate")] 
        value: BuildableValue,
    },
}
impl BuildableSongAction {
    pub fn into_action(
        self, 
        values: &mut dyn Reflect, 
        passed_in: Option<&TatakuValue>,
    ) -> Option<SongAction> {
        match self {
            Self::Play => Some(SongAction::Play),
            Self::Pause => Some(SongAction::Pause),
            Self::Toggle => Some(SongAction::Toggle),
            Self::Restart => Some(SongAction::Restart),
            Self::PushQueue => Some(SongAction::Set(SongSetAction::PushQueue)),
            Self::PopQueue { value } 
                => Some(SongAction::Set(SongSetAction::PopQueue(
                    value.resolve(values, passed_in)?,
                ))),

            Self::Seek { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(SongAction::SeekBy),

            Self::SetPosition { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(SongAction::SetPosition),

            Self::SetRate { value } => value
                .resolve(values, passed_in)
                .and_then(|n| n.as_f32())
                .map(SongAction::SetRate),
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


#[derive(Clone, Debug, PartialEq, Default)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildableSongPlayData {
    #[serde(alias="@play", default)] pub play: Option<BuildableValueTag>,
    #[serde(alias="@restart", default)] pub restart: Option<BuildableValueTag>,
    #[serde(alias="@position", default)] pub position: Option<BuildableValueTag>,
    #[serde(alias="@volume", default)] pub rate: Option<BuildableValueTag>,
    #[serde(alias="@rate", default)] pub volume: Option<BuildableValueTag>,
}
impl BuildableSongPlayData {
    pub fn resolve(
        &self,
        values: &mut dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<SongPlayData> {
        Some(SongPlayData { 
            play: self
                .play
                .as_ref()
                .and_then(|a| a
                    .resolve(values, passed_in)
                    .map(|i| i.as_bool())
                )
                .unwrap_or_default(),

            restart: self
                .restart
                .as_ref()
                .and_then(|a| a
                    .resolve(values, passed_in)
                    .map(|i| i.as_bool())
                )
                .unwrap_or_default(),

            position: self
                .position
                .as_ref()
                .and_then(|a| a
                    .resolve(values, passed_in)
                    .and_then(|i| i.as_f32())
                ), 

            rate: self
                .rate
                .as_ref()
                .and_then(|a| a
                    .resolve(values, passed_in)
                    .and_then(|i| i.as_f32())
                ), 

            volume: self
                .volume
                .as_ref()
                .and_then(|a| a
                    .resolve(values, passed_in)
                    .and_then(|i| i.as_f32())
                ), 
        })
    }
}