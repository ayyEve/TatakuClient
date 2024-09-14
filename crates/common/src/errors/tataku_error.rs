use std::{ fmt::Display, io::Error as IOError };

use serde_json::Error as JsonError;
use tataku_common::*;

use super::*;
pub type TatakuResult<T=()> = Result<T, TatakuError>;

#[derive(Debug)]
#[allow(dead_code, unused)]
pub enum TatakuError {
    Beatmap(BeatmapError),
    GameMode(GameModeError),
    IO(IOError),
    Serde(JsonError),

    Audio(AudioError),
    // #[cfg(feature = "graphics")]
    Image(image::ImageError),
    Graphics(GraphicsError),

    String(String),
    SerializationError(SerializationError),
    ReqwestError(reqwest::Error),
    DownloadError(DownloadError),

    // ShuntingYardError(ShuntingYardError),

    #[cfg(feature = "ui")]
    Lua(rlua::Error),

    ReflectError(ReflectError<'static>)
}
impl TatakuError {
    pub fn from_err(e: impl std::error::Error) -> Self {
        Self::String(format!("{e}"))
    }
}
impl From<&str> for TatakuError {
    fn from(value: &str) -> Self {
        TatakuError::String(value.to_owned())
    }
}


impl Display for TatakuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Beatmap(e) => write!(f, "{:?}", e),
            Self::Serde(e) => write!(f, "{:?}", e),
            Self::IO(e) => write!(f, "{}", e),
            // #[cfg(feature = "graphics")]
            Self::Image(e) => write!(f, "{:?}", e),
            Self::Audio(e) => write!(f, "{:?}", e),
            Self::String(e) => write!(f, "{:?}", e),
            Self::GameMode(e) => write!(f, "{:?}", e),
            Self::SerializationError(e) => write!(f, "{:?}", e),
            Self::ReqwestError(e) => write!(f, "{:?}", e),
            Self::DownloadError(e) => write!(f, "{:?}", e),
            Self::Graphics(e) => write!(f, "{:?}", e),
            // Self::ShuntingYardError(e) => write!(f, "{:?}", e),
            #[cfg(feature = "ui")]
            Self::Lua(e) => write!(f, "{:?}", e),
            Self::ReflectError(e) => write!(f, "{:?}", e),
        }
    }
}


impl From<JsonError> for TatakuError {
    fn from(e: JsonError) -> Self {Self::Serde(e)}
}
impl From<IOError> for TatakuError {
    fn from(e: IOError) -> Self {Self::IO(e)}
}
// #[cfg(feature = "graphics")]
impl From<image::ImageError> for TatakuError {
    fn from(e: image::ImageError) -> Self {Self::Image(e)}
}
impl From<AudioError> for TatakuError {
    fn from(e: AudioError) -> Self {Self::Audio(e)}
}

impl From<BeatmapError> for TatakuError {
    fn from(e: BeatmapError) -> Self {Self::Beatmap(e)}
}
impl From<String> for TatakuError {
    fn from(e: String) -> Self {Self::String(e)}
}
impl From<GameModeError> for TatakuError {
    fn from(e: GameModeError) -> Self {Self::GameMode(e)}
}
impl From<SerializationError> for TatakuError {
    fn from(e: SerializationError) -> Self {Self::SerializationError(e)}
}
impl From<reqwest::Error> for TatakuError {
    fn from(e: reqwest::Error) -> Self {Self::ReqwestError(e)}
}
impl From<DownloadError> for TatakuError {
    fn from(e: DownloadError) -> Self {Self::DownloadError(e)}
}
// impl From<ShuntingYardError> for TatakuError {
//     fn from(value: ShuntingYardError) -> Self { Self::ShuntingYardError(value) }
// }

#[cfg(feature = "ui")]
impl From<rlua::Error> for TatakuError {
    fn from(value: rlua::Error) -> Self { Self::Lua(value) }
}
impl From<GraphicsError> for TatakuError {
    fn from(value: GraphicsError) -> Self { Self::Graphics(value) }
}
impl<'a> From<ReflectError<'a>> for TatakuError {
    fn from(value: ReflectError<'a>) -> Self {
        Self::ReflectError(value.to_owned())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GameModeError {
    NotImplemented,
    UnknownGameMode
}