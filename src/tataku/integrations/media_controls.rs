use crate::prelude::*;
use souvlaki::{ MediaControlEvent, MediaMetadata, SeekDirection, MediaPlayback, MediaPosition };

/// how long to wait between events of the same type before handling the next in ms (should help with windows event spam)
const MINIMUM_WAIT_BETWEEN_EVENTS:f32 = 100.0;

/// helper for interfacing with the os media controls
pub struct MediaControlHelper {
    event_receiver: AsyncUnboundedReceiver<MediaControlEvent>,
    internal_event_sender: AsyncUnboundedSender<MediaControlEvent>,

    // when it was received, the event, was it handled upstream?
    last_event: LastEventHelper,
    event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,
    last_state: MediaPlaybackState,

    current_metadata: MediaControlMetadata,
    controls_enabled: bool,
}

impl MediaControlHelper {
    pub fn new(event_sender: AsyncUnboundedSender<MediaControlHelperEvent>) -> Self {
        let (sender, receiver) = async_unbounded_channel();
        let controls_enabled = get_settings!().integrations.media_controls;
        if controls_enabled {
            Self::bind(sender.clone());
        }

        Self {
            event_receiver: receiver,
            internal_event_sender: sender,
            event_sender,
            controls_enabled,
            current_metadata: MediaControlMetadata::default(),

            last_event: LastEventHelper::default(),
            last_state: MediaPlaybackState::Stopped
        }
    }

    pub async fn update(&mut self, song_state: MediaPlaybackState, enabled: bool) {
        if enabled != self.controls_enabled {
            self.controls_enabled = enabled;
            if enabled {
                Self::bind(self.internal_event_sender.clone());
                Self::set_metadata(&self.current_metadata);
            }
        }

        // update events 
        if let Ok(event) = self.event_receiver.try_recv() {
            if event != self.last_event.event || self.last_event.time.as_millis() >= MINIMUM_WAIT_BETWEEN_EVENTS {
                let _ = self.event_sender.send(event.clone().into());
                self.last_event = LastEventHelper::new(event);
            }
        }

        let (last_state, last_elapsed) = match self.last_state {
            MediaPlaybackState::Playing(e) => (0, e),
            MediaPlaybackState::Paused(e) => (1, e),
            MediaPlaybackState::Stopped => (2, 0.0),
        };
        let (current_state, current_elapsed) = match song_state {
            MediaPlaybackState::Playing(e) => (0, e),
            MediaPlaybackState::Paused(e) => (1, e),
            MediaPlaybackState::Stopped => (2, 10000.0),
        };

        // only update once every Xms
        if last_state == current_state && current_elapsed - last_elapsed < 1000.0 { return }
        self.last_state = song_state;

        // update the playback state
        Self::set_playback(song_state.into());
    }

    pub fn update_info(&mut self, map: &Option<Arc<BeatmapMeta>>, duration: f32) {
        self.current_metadata = map.as_ref().map(|map| MediaControlMetadata {
            title: Some(map.title.clone()),
            artist: Some(map.artist.clone()),
            cover_url: Some("file://".to_owned() + Path::new(&map.image_filename).canonicalize().map(|p|p.to_string_lossy().to_string()).unwrap_or_default().trim_start_matches("\\\\?\\")),
            duration: Some(duration),
        }).unwrap_or_default();
        
        Self::set_metadata(&self.current_metadata);
    }

    fn bind(sender: AsyncUnboundedSender<MediaControlEvent>) {
        if !get_settings!().integrations.media_controls { return }
        let controls = GameWindow::get_media_controls();
        let _ = controls.lock().attach(move |event|{let _ = sender.send(event);});
    }
    pub fn set_metadata(meta: &MediaControlMetadata) {
        if !get_settings!().integrations.media_controls { return }

        fn s(a: &Option<String>) -> Option<&str> {
            a.as_ref().map(|x| &**x)
        }

        let info = MediaMetadata {
            title: s(&meta.title),
            artist: s(&meta.artist),
            album: None,
            cover_url: s(&meta.cover_url),
            duration: meta.duration.map(|d|Duration::from_millis(d as u64))
        };

        // duration: Some(Duration::from_millis(duration as u64))
        if let Err(e) = GameWindow::get_media_controls().lock().set_metadata(info) {
            warn!("Error setting metadata: {e:?}");
        }
    }
    pub fn set_playback(state: MediaPlayback) {
        if !get_settings!().integrations.media_controls { return }

        tokio::task::spawn_blocking(move || {
            if let Err(e) = GameWindow::get_media_controls().lock().set_playback(state) {
                warn!("Error setting playback state: {e:?}");
            }
        });
    }
}

impl Drop for MediaControlHelper {
    fn drop(&mut self) {
        Self::set_metadata(&Default::default());
        Self::set_playback(MediaPlayback::Stopped);
    }
}

struct LastEventHelper {
    time: Instant,
    event: MediaControlEvent,
}
impl LastEventHelper {
    fn new(event: MediaControlEvent) -> Self {
        Self {
            time: Instant::now(),
            event,
        }
    }
}
impl Default for LastEventHelper {
    fn default() -> Self {
        Self {
            time: Instant::now(),
            event: MediaControlEvent::Pause,
        }
    }
}


#[derive(Clone, Debug)]
pub enum MediaControlHelperEvent {
    Play,
    Pause,
    Stop,
    Toggle,

    Next,
    Previous,

    SeekForward,
    SeekBackward,
    SeekForwardBy(f32),
    SeekBackwardBy(f32),
    SetPosition(f32),
    OpenUri(String),
    Raise,
    Quit,
}
impl From<MediaControlEvent> for MediaControlHelperEvent {
    fn from(value: MediaControlEvent) -> Self {
        match value {
            MediaControlEvent::Play => Self::Play,
            MediaControlEvent::Pause => Self::Pause,
            MediaControlEvent::Toggle => Self::Toggle,
            MediaControlEvent::Next => Self::Next,
            MediaControlEvent::Previous => Self::Previous,
            MediaControlEvent::Stop => Self::Stop,
            MediaControlEvent::Seek(SeekDirection::Forward) => Self::SeekForward,
            MediaControlEvent::Seek(SeekDirection::Backward) => Self::SeekBackward,
            MediaControlEvent::SeekBy(SeekDirection::Forward, d) => Self::SeekForwardBy(d.as_secs_f32() * 1000.0),
            MediaControlEvent::SeekBy(SeekDirection::Backward, d) => Self::SeekBackwardBy(d.as_secs_f32() * 1000.0),
            MediaControlEvent::SetPosition(pos) => Self::SetPosition(pos.0.as_secs_f32() * 1000.0),
            MediaControlEvent::OpenUri(uri) => Self::OpenUri(uri),
            MediaControlEvent::Raise => Self::Raise,
            MediaControlEvent::Quit => Self::Quit,
        }
    }
}


#[derive(PartialEq, Copy, Clone, Debug)]
pub enum MediaPlaybackState {
    Playing(f32),
    Paused(f32),
    Stopped,
}
impl Into<MediaPlayback> for MediaPlaybackState {
    fn into(self) -> MediaPlayback {
        match self {
            MediaPlaybackState::Playing(time) => MediaPlayback::Playing { progress: Some(MediaPosition(Duration::from_millis(time as u64))) },
            MediaPlaybackState::Paused(time) => MediaPlayback::Paused { progress: Some(MediaPosition(Duration::from_millis(time as u64))) },
            MediaPlaybackState::Stopped => MediaPlayback::Stopped,
        }
    }
}


#[derive(Clone, Debug, Default)]
pub struct MediaControlMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub cover_url: Option<String>,
    pub duration: Option<f32>,
}