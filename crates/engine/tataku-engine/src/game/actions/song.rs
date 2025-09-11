use crate::*;
use tataku_graphics::FFTHook;
use tataku_client_common::prelude::*;

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

    /// Set the song volume
    SetVolume(f32),

    /// Set the playback rate of the current song
    SetRate(f32),

    /// Change the current song. 
    /// 
    /// You probably don't want to touch this in custom code
    Set(SongSetAction),

    /// Add a hook to fft data
    #[cfg(feature="graphics")]
    HookFFT(Weak<FFTHook>),
}

#[derive(Clone, Debug)]
pub enum SongSetAction {
    /// Push the current song to the play queue
    PushQueue,
    
    /// Pop the latest song from the play queue
    /// 
    /// Will only run if there is something in the queue
    PopQueue(SongPlayData),

    /// Remove the current song, setting it to none
    Remove,

    /// Play a file from the disk
    FromFile(ArcStr, SongPlayData),

    /// Play from bytes
    FromData(Vec<u8>, ArcStr, SongPlayData),
}

impl From<SongAction> for actions::Action {
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
