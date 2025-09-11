use crate::prelude::*;
pub type TatakuResult<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    Beatmap(errors::beatmap::BeatmapError),
    GameMode(errors::game_mode::GameModeError),
    IO(std::io::Error),
    Serde(serde_json::Error),

    Audio(errors::audio::AudioError),
    // #[cfg(feature = "graphics")]
    Image(image::ImageError),
    Graphics(errors::graphics::GraphicsError),

    String(String),
    SerializationError(common::serialization::SerializationError),
    ReqwestError(reqwest::Error),
    DownloadError(errors::download::DownloadError),

    DiffCalcError(errors::diffcalc::DiffCalcError),

    #[from(skip)]
    ReflectError(common::reflect::ReflectError<'static>),

    CustomMenuError(errors::custom_menu::CustomMenuError),
}
impl Error {
    pub fn from_err(e: impl std::error::Error) -> Self {
        Self::String(format!("{e}"))
    }
    #[allow(clippy::needless_pass_by_value, reason = "conversion function")]
    pub fn from_boxed_err(e: Box<dyn std::error::Error>) -> Self {
        Self::String(format!("{e}"))
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::String(value.to_owned())
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Beatmap(e) => write!(f, "{e:?}"),
            Self::Serde(e) => write!(f, "{e:?}"),
            Self::IO(e) => write!(f, "{e}"),
            Self::Image(e) => write!(f, "{e:?}"),
            Self::Audio(e) => write!(f, "{e:?}"),
            Self::String(e) => write!(f, "{e:?}"),
            Self::GameMode(e) => write!(f, "{e:?}"),
            Self::SerializationError(e) => write!(f, "{e:?}"),
            Self::ReqwestError(e) => write!(f, "{e:?}"),
            Self::DownloadError(e) => write!(f, "{e:?}"),
            Self::Graphics(e) => write!(f, "{e:?}"),
            Self::DiffCalcError(e) => write!(f, "{e:?}"),
            
            Self::ReflectError(e) => write!(f, "{e:?}"),
            Self::CustomMenuError(e) => write!(f, "{e:?}"),
        }
    }
}

impl<'a> From<common::reflect::ReflectError<'a>> for Error {
    fn from(value: common::reflect::ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}

