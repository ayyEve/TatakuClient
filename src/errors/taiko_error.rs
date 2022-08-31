use std::{fmt::Display, io::Error as IOError};

use image::ImageError;

#[cfg(feature="bass_audio")]
use bass_rs::prelude::BassError;
use serde_json::Error as JsonError;
use tataku_common::SerializationError;

use super::*;

pub type TatakuResult<T> = Result<T, TatakuError>;

#[derive(Debug)]
#[allow(dead_code, unused)]
pub enum TatakuError {
    Beatmap(BeatmapError),
    GameMode(GameModeError),
    IO(IOError),
    Serde(JsonError),

    Audio(AudioError),
    Image(ImageError),
    GlError(GlError),

    String(String),
    SerializationError(SerializationError),
    ReqwestError(reqwest::Error),
}
impl Display for TatakuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TatakuError::Beatmap(e) => write!(f, "{:?}", e),
            TatakuError::Serde(e) => write!(f, "{:?}", e),
            TatakuError::IO(e) => write!(f, "{}", e),
            TatakuError::Image(e) => write!(f, "{:?}", e),
            TatakuError::Audio(e) => write!(f, "{:?}", e),
            TatakuError::String(e) => write!(f, "{:?}", e),
            TatakuError::GlError(e) => write!(f, "{:?}", e),
            TatakuError::GameMode(e) => write!(f, "{:?}", e),
            TatakuError::SerializationError(e) => write!(f, "{:?}", e),
            TatakuError::ReqwestError(e) => write!(f, "{:?}", e),
        }
    }
}


impl From<JsonError> for TatakuError {
    fn from(e: JsonError) -> Self {TatakuError::Serde(e)}
}
impl From<IOError> for TatakuError {
    fn from(e: IOError) -> Self {TatakuError::IO(e)}
}
impl From<ImageError> for TatakuError {
    fn from(e: ImageError) -> Self {TatakuError::Image(e)}
}
impl From<AudioError> for TatakuError {
    fn from(e: AudioError) -> Self {TatakuError::Audio(e)}
}
#[cfg(feature="bass_audio")]
impl From<BassError> for TatakuError {
    fn from(e: BassError) -> Self {TatakuError::Audio(AudioError::BassError(e))}
}

impl From<BeatmapError> for TatakuError {
    fn from(e: BeatmapError) -> Self {TatakuError::Beatmap(e)}
}
impl From<GlError> for TatakuError {
    fn from(e: GlError) -> Self {TatakuError::GlError(e)}
}
impl From<String> for TatakuError {
    fn from(e: String) -> Self {TatakuError::String(e)}
}
impl From<GameModeError> for TatakuError {
    fn from(e: GameModeError) -> Self {TatakuError::GameMode(e)}
}
impl From<SerializationError> for TatakuError {
    fn from(e: SerializationError) -> Self {TatakuError::SerializationError(e)}
}
impl From<reqwest::Error> for TatakuError {
    fn from(e: reqwest::Error) -> Self {TatakuError::ReqwestError(e)}
}


#[derive(Clone, Copy, Debug)]
pub enum GameModeError {
    NotImplemented,
    UnknownGameMode
}