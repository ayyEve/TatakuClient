use std::{fmt::Display, io::Error as IOError};

use image::ImageError;

#[cfg(feature="bass_audio")]
use bass_rs::prelude::BassError;
use serde_json::Error as JsonError;

use super::*;

pub type TaikoResult<T> = Result<T, TaikoError>;

#[derive(Debug)]
#[allow(dead_code, unused)]
pub enum TaikoError {
    Beatmap(BeatmapError),
    GameMode(GameModeError),
    IO(IOError),
    Serde(JsonError),

    Audio(AudioError),
    Image(ImageError),
    GlError(GlError),

    String(String),
}
impl Display for TaikoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TaikoError::Beatmap(e) => write!(f, "{:?}", e),
            TaikoError::Serde(e) => write!(f, "{:?}", e),
            TaikoError::IO(e) => write!(f, "{}", e),
            TaikoError::Image(e) => write!(f, "{:?}", e),
            TaikoError::Audio(e) => write!(f, "{:?}", e),
            TaikoError::String(e) => write!(f, "{:?}", e),
            TaikoError::GlError(e) => write!(f, "{:?}", e),
            TaikoError::GameMode(e) => write!(f, "{:?}", e),
        }
    }
}


impl From<JsonError> for TaikoError {
    fn from(e: JsonError) -> Self {TaikoError::Serde(e)}
}
impl From<IOError> for TaikoError {
    fn from(e: IOError) -> Self {TaikoError::IO(e)}
}
impl From<ImageError> for TaikoError {
    fn from(e: ImageError) -> Self {TaikoError::Image(e)}
}
impl From<AudioError> for TaikoError {
    fn from(e: AudioError) -> Self {TaikoError::Audio(e)}
}
#[cfg(feature="bass_audio")]
impl From<BassError> for TaikoError {
    fn from(e: BassError) -> Self {TaikoError::Audio(AudioError::BassError(e))}
}

impl From<BeatmapError> for TaikoError {
    fn from(e: BeatmapError) -> Self {TaikoError::Beatmap(e)}
}
impl From<GlError> for TaikoError {
    fn from(e: GlError) -> Self {TaikoError::GlError(e)}
}
impl From<String> for TaikoError {
    fn from(e: String) -> Self {TaikoError::String(e)}
}
impl From<GameModeError> for TaikoError {
    fn from(e: GameModeError) -> Self {TaikoError::GameMode(e)}
}

#[derive(Clone, Copy, Debug)]
pub enum GameModeError {
    NotImplemented,
    UnknownGameMode
}