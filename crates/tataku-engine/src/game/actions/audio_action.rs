use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct AudioAction {
    pub id: String,
    pub action: AudioActionType,
}
impl AudioAction {
    pub fn new(id: impl Into<String>, action: impl Into<AudioActionType>) -> Self {
        Self {
            id: id.into(),
            action: action.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AudioActionType {
    Play {
        /// volume as a %
        volume: f32,

        /// should the sound repeat?
        repeat: bool,

        /// should the sound restart?
        restart: bool,
    },

    /// stop playing a sound (mainly used if sound is repeat)
    Stop,

    /// load an audio sample
    Load {
        /// list of possible samples to load for the provided id.
        /// samples will attempt to be loaded in order, stopping at the first successful load
        /// this enabled loading fallback sounds
        list: Vec<AudioLoadData>,
    },

    /// Unload the audio, freeing the stream
    Unload
}

#[derive(Clone, Debug)]
pub struct AudioLoadData {
    /// relative path to the audio file
    pub path: String,
    
    /// where to load the audio from
    pub source: HitsoundSource,
}
impl AudioLoadData {
    pub fn new(path: impl Into<String>, source: HitsoundSource) -> Self {
        Self {
            path: path.into(),
            source
        }
    }

    pub fn new_multi_source(path: impl Into<String>, prefix: Option<impl Into<String>>, sources: &[HitsoundSource]) -> Vec<Self> {
        let path: String = path.into();
        let mut list = sources
            .iter()
            .copied()
            .map(|source| Self::new(path.clone(), source))
            .collect::<Vec<_>>();

        if let Some(prefix) = prefix {
            let prefix= prefix.into();
            list = list
            .into_iter()
                .chain(
                sources
                    .iter()
                    .copied()
                    .map(|source| {
                        // let p = patj // TODO: account for directories in path
                        Self::new(format!("{prefix}-{path}"), source)
                    })
                )
                .collect()
        }

        list
    }
}
impl From<AudioAction> for TatakuAction {
    fn from(value: AudioAction) -> Self {
        Self::Audio(value)
    }
}



#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HitsoundSource {
    Skin,
    Beatmap,
    Default
}
