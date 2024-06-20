use std::{fmt::Display, io::Error as IOError};

use image::ImageError;

use serde_json::Error as JsonError;
use tataku_common::SerializationError;
use crate::prelude::ShuntingYardError;

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
    Image(ImageError),
    Graphics(GraphicsError),

    String(String),
    SerializationError(SerializationError),
    ReqwestError(reqwest::Error),
    DownloadError(DownloadError),

    ShuntingYardError(ShuntingYardError),

    Lua(rlua::Error)
}
impl TatakuError {
    pub fn from_err(e: impl std::error::Error) -> Self {
        Self::String(format!("{e}"))
    }
}


impl Display for TatakuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Beatmap(e) => write!(f, "{:?}", e),
            Self::Serde(e) => write!(f, "{:?}", e),
            Self::IO(e) => write!(f, "{}", e),
            Self::Image(e) => write!(f, "{:?}", e),
            Self::Audio(e) => write!(f, "{:?}", e),
            Self::String(e) => write!(f, "{:?}", e),
            Self::GameMode(e) => write!(f, "{:?}", e),
            Self::SerializationError(e) => write!(f, "{:?}", e),
            Self::ReqwestError(e) => write!(f, "{:?}", e),
            Self::DownloadError(e) => write!(f, "{:?}", e),
            Self::Graphics(e) => write!(f, "{:?}", e),
            Self::ShuntingYardError(e) => write!(f, "{:?}", e),
            Self::Lua(e) => write!(f, "{:?}", e),
        }
    }
}


impl From<JsonError> for TatakuError {
    fn from(e: JsonError) -> Self {Self::Serde(e)}
}
impl From<IOError> for TatakuError {
    fn from(e: IOError) -> Self {Self::IO(e)}
}
impl From<ImageError> for TatakuError {
    fn from(e: ImageError) -> Self {Self::Image(e)}
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
impl From<ShuntingYardError> for TatakuError {
    fn from(value: ShuntingYardError) -> Self { Self::ShuntingYardError(value) }
}

impl From<rlua::Error> for TatakuError {
    fn from(value: rlua::Error) -> Self { Self::Lua(value) }
}
impl From<GraphicsError> for TatakuError {
    fn from(value: GraphicsError) -> Self { Self::Graphics(value) }
}


#[derive(Clone, Copy, Debug)]
pub enum GameModeError {
    NotImplemented,
    UnknownGameMode
}