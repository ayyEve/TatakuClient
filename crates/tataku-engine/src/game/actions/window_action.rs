use crate::prelude::*;

#[derive(Debug)]
pub enum WindowAction {
    MediaControlAction(MediaControlAction),
}
impl From<WindowAction> for TatakuAction {
    fn from(value: WindowAction) -> Self {
        Self::WindowAction(value)
    }
}

#[derive(Debug)]
pub enum MediaControlAction {
    Attach,
    Detatch,
    SetPlayback(MediaPlaybackState),
    SetMetadata(MediaControlMetadata),
}

impl From<MediaControlAction> for TatakuAction {
    fn from(value: MediaControlAction) -> Self {
        Self::WindowAction(WindowAction::MediaControlAction(value))
    }
}


#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MediaPlaybackState {
    Playing(f32),
    Paused(f32),
    Stopped,
}
#[cfg(feature="graphics")]
impl Into<souvlaki::MediaPlayback> for MediaPlaybackState {
    fn into(self) -> souvlaki::MediaPlayback {
        match self {
            MediaPlaybackState::Playing(time) => souvlaki::MediaPlayback::Playing { progress: Some(souvlaki::MediaPosition(Duration::from_millis(time as u64))) },
            MediaPlaybackState::Paused(time) => souvlaki::MediaPlayback::Paused { progress: Some(souvlaki::MediaPosition(Duration::from_millis(time as u64))) },
            MediaPlaybackState::Stopped => souvlaki::MediaPlayback::Stopped,
        }
    }
}


#[derive(Clone, Debug, Default)]
pub struct MediaControlMetadata {
    pub title: Option<Cow<'static, str>>,
    pub artist: Option<Cow<'static, str>>,
    pub cover_url: Option<Cow<'static, str>>,
    pub duration: Option<f32>,
}
#[cfg(feature="graphics")]
impl<'a> Into<souvlaki::MediaMetadata<'a>> for &'a MediaControlMetadata {
    fn into(self) -> souvlaki::MediaMetadata<'a> {
        souvlaki::MediaMetadata {
            title: self.title.as_ref().map(Cow::as_ref),
            album: None,
            artist: self.artist.as_ref().map(Cow::as_ref),
            cover_url: self.cover_url.as_ref().map(Cow::as_ref),
            duration: self.duration.map(|ms| Duration::from_secs_f32(ms * 1000.0)),
        }
    }
}