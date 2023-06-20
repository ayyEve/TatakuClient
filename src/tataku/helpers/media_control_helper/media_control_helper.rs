use crate::prelude::*;
use souvlaki::{ MediaControlEvent, MediaMetadata, SeekDirection, MediaPlayback, MediaPosition };

/// how long to wait between events of the same type before handling the next in ms (should help with windows event spam)
const MINIMUM_WAIT_BETWEEN_EVENTS:f32 = 100.0;

/// helper for interfacing with the os media controls
pub struct MediaControlHelper {
    event_receiver: AsyncUnboundedReceiver<MediaControlEvent>,

    // when it was received, the event, was it handled upstream?
    last_event: LastEventHelper,

    event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,

    last_state: MediaPlaybackState,
}

impl MediaControlHelper {
    pub fn new(event_sender: AsyncUnboundedSender<MediaControlHelperEvent>) -> Self {
        let (sender, receiver) = async_unbounded_channel();
        
        let controls = GameWindow::get_media_controls();
        let _ = controls.lock().attach(move |event|{let _ = sender.send(event);});

        Self {
            event_receiver: receiver,
            event_sender,

            last_event: LastEventHelper::default(),
            last_state: MediaPlaybackState::Stopped
        }
    }

    pub async fn update(&mut self, song_state: MediaPlaybackState) {
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
        if last_state == current_state && current_elapsed - last_elapsed < 100.0 { return }
        self.last_state = song_state;

        // update the playback state
        tokio::task::spawn_blocking(move || {
            let controls = GameWindow::get_media_controls();
            let mut lock = controls.lock();
            if let Err(e) = lock.set_playback(song_state.into()) {
                warn!("Error setting playback state: {e:?}");
            }
        });
    }

    pub fn update_info(&self, map: &Option<Arc<BeatmapMeta>>, duration: f32) {
        let mut img_url = String::new();
        let mut info = if let Some(map) = map {
            img_url = "file://".to_owned() + Path::new(&map.image_filename).canonicalize().map(|p|p.to_string_lossy().to_string()).unwrap_or_default().trim_start_matches("\\\\?\\");
            info!("Using media url: {img_url}");
            MediaMetadata {
                title: Some(&map.title),
                artist: Some(&map.artist),
                album: None,
                cover_url: None,
                duration: Some(Duration::from_millis(duration as u64))
            }
        } else {
            MediaMetadata::default()
        };
        if !img_url.is_empty() {
            info.cover_url = Some(&img_url);
        }
        
        let controls = GameWindow::get_media_controls();
        let mut lock = controls.lock();
        if let Err(e) = lock.set_metadata(info) {
            warn!("Error setting metadata: {e:?}");
        }
    }
}

impl Drop for MediaControlHelper {
    fn drop(&mut self) {
        let controls = GameWindow::get_media_controls();
        let mut lock = controls.lock();
        let _ = lock.set_metadata(MediaMetadata::default());
        let _ = lock.set_playback(MediaPlayback::Stopped);
        // for some reason on windows if you detach here, it causes issues when you re-attach. i have no idea why
        #[cfg(not(target_os="windows"))] 
        let _ = lock.detach();
    }
}

// helper for converting media control events
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

impl Into<MediaPlayback> for MediaPlaybackState {
    fn into(self) -> MediaPlayback {
        match self {
            MediaPlaybackState::Playing(time) => MediaPlayback::Playing { progress: Some(MediaPosition(Duration::from_millis(time as u64))) },
            MediaPlaybackState::Paused(time) => MediaPlayback::Paused { progress: Some(MediaPosition(Duration::from_millis(time as u64))) },
            MediaPlaybackState::Stopped => MediaPlayback::Stopped,
        }
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
