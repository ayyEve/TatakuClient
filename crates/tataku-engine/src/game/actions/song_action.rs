use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum SongAction {
    /// Play/Resume the current song
    Play,

    /// Restart the current song
    Restart,

    /// Pause the current song
    Pause,

    /// Stop the current song
    Stop,

    /// Play/pause the current song
    Toggle,

    /// Seek by the specified amount (negative means seek backwards)
    SeekBy(f32),

    /// Set the position of the current song (in ms)
    SetPosition(f32),

    /// set the song volume
    SetVolume(f32),

    /// set the playback rate of the current song
    SetRate(f32),

    /// change the current song. you probably dont want to touch this in custom code
    Set(SongMenuSetAction),
}

#[derive(Clone, Debug)]
pub enum SongMenuSetAction {
    /// Push the current song to the play queue
    PushQueue,
    
    /// Pop the latest song from the play queue and play it
    PopQueue,

    /// remove the current song, setting it to none
    Remove,

    /// Play a file from the disk
    FromFile(String, SongPlayData),

    /// Play from bytes
    FromData(Vec<u8>, String, SongPlayData),
}

impl From<SongAction> for TatakuAction {
    fn from(value: SongAction) -> Self { Self::Song(value) }
}


#[derive(Copy, Clone, Debug, Default)]
pub struct SongPlayData {
    pub play: bool,
    pub restart: bool,
    pub position: Option<f32>,
    pub rate: Option<f32>,
    pub volume: Option<f32>,
}