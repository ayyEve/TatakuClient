use tataku_audio::prelude::*;
use tataku_engine::prelude::*;
use souvlaki::{ 
    MediaControlEvent, 
    MediaControls, 
    MediaPlayback, 
    MediaPosition, 
    PlatformConfig, 
    SeekDirection,
};

use tokio::sync::mpsc::unbounded_channel as event_channel;
type EventSender = tokio::sync::mpsc::UnboundedSender<MediaControlEvent>;
type EventReceiver = tokio::sync::mpsc::UnboundedReceiver<MediaControlEvent>;

/// how long to wait between events of the same type before handling the next in ms (should help with windows event spam)
const MINIMUM_WAIT_BETWEEN_EVENTS:f32 = 100.0;

/// how much to seek by if no seek amount is provided
const DEFAULT_SEEK_AMOUNT: f32 = 500.0;

pub struct MediaControlsIntegration {
    sender: EventSender,
    receiver: EventReceiver,
    media_controls: Option<MediaControls>,


    enabled: bool, 
    attached: bool,
    last_event: LastEventHelper,
}
impl MediaControlsIntegration {
    fn build() -> TatakuResult<Box<dyn TatakuIntegration>> {
        let (sender, receiver) = event_channel();
        Ok(Box::new(Self {
            sender,
            receiver,
            media_controls: None,

            enabled: false,
            attached: false,
            last_event: LastEventHelper::default(),
        }))
    }

    pub fn builder() -> TatakuIntegrationBuilder {
        TatakuIntegrationBuilder {
            name: "media_controls_integration",
            build: Self::build,
        }
    }
}
impl TatakuIntegration for MediaControlsIntegration {
    fn name(&self) -> CowStr { "media_controls_integration".into() }
    
    #[allow(unused)]
    #[cfg(not(feature="graphics"))]
    fn init(
        &mut self, 
        #[cfg(feature="graphics")] 
        window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> TatakuResult<()> {
        Ok(())
    }
    
    #[allow(unused)]
    #[cfg(feature="graphics")] 
    fn init(
        &mut self, 
        #[cfg(feature="graphics")] 
        window_handle: raw_window_handle::WindowHandle<'_>,
    ) -> TatakuResult<()> {

        if self.media_controls.is_some() { return Ok(()) }

        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        // TODO: need to store this somehow? might be worth putting on a new std thread so we can store it and operations dont lock the game thread
        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::RawWindowHandle;
            let window_ptr = match window_handle.as_raw() {
                RawWindowHandle::Win32(h) => h.hwnd.get() as *mut std::ffi::c_void,
                RawWindowHandle::WinRt(h) => h.core_window.as_ptr(),
                _ => unreachable!(),
            };

            Some(window_ptr)
        };

        self.media_controls = Some(MediaControls::new(PlatformConfig {
            dbus_name: "tataku.player",
            display_name: "Tataku!",
            hwnd,
        }).map_err(map_err)?);

        Ok(())
    }
    
    fn check_enabled(
        &mut self, 
        settings: &Settings,
    ) -> TatakuResult<()> {
        let Some(controls) = self.media_controls.as_mut() else { return Ok(()) };
        self.enabled = settings.integrations.media_controls;

        if self.enabled && !self.attached {
            self.attached = true;
            let sender = self.sender.clone();
            controls
                .attach(move |e| sender.send(e).unwrap())
                .map_err(map_err)?;
            trace!("media controls attached");
        } else if !self.enabled && self.attached {
            trace!("detaching media controls");
            self.attached = false;
            controls
                .detach()
                .map_err(map_err)?;
        }

        Ok(())
    }

    fn handle_event(
        &mut self, 
        event: &TatakuIntegrationEvent, 
        _values: &dyn Reflect,
        _actions: &mut ActionQueue,
    ) {
        if !self.enabled { return }
        let Some(controls) = self.media_controls.as_mut() else { return };

        #[allow(clippy::single_match, reason = "its fine")]
        match event {
            TatakuIntegrationEvent::SongChanged { 
                artist, 
                title, 
                duration,
                ..
            } => {
                if let Err(e) = controls.set_metadata(souvlaki::MediaMetadata { 
                    artist: Some(artist), 
                    title: Some(title), 
                    album: None, 
                    cover_url: None, 
                    duration: Some(Duration::from_secs_f32(*duration / 1000.0))
                }) {
                    error!("error setting metadata: {e:?}");
                }
            }        
            _ => {}
        }
    }

    fn update(
        &mut self, 
        values: &mut dyn Reflect, 
        actions: &mut ActionQueue,
    ) {
        if !self.enabled { return }
        let Some(controls) = self.media_controls.as_mut() else { return };

        let Ok(position) = values.reflect_get::<f32>("song.position") else { return };
        let Ok(state) = values.reflect_get::<AudioState>("song.state") else { return };
        let progress = MediaPosition(Duration::from_secs_f32(*position / 1000.0));

        let playback = match *state {
            AudioState::Playing => MediaPlayback::Playing { progress: Some(progress) },
            AudioState::Paused => MediaPlayback::Paused { progress: Some(progress) },
            AudioState::Stopped => MediaPlayback::Stopped,
            AudioState::Unknown => MediaPlayback::Stopped,
        };
        controls.set_playback(playback).unwrap();
        

        if let Ok(event) = self.receiver.try_recv() {
            if event == self.last_event.event || self.last_event.time.as_millis() < MINIMUM_WAIT_BETWEEN_EVENTS { return }

            self.last_event = LastEventHelper::new(event.clone());

            match event {
                MediaControlEvent::Play => actions.push(SongAction::Play),
                MediaControlEvent::Pause => actions.push(SongAction::Pause),
                MediaControlEvent::Toggle => actions.push(SongAction::Toggle),
                MediaControlEvent::Next => actions.push(BeatmapAction::Next),
                MediaControlEvent::Previous => actions.push(BeatmapAction::Previous(MapActionIfNone::ContinueCurrent)),
                MediaControlEvent::Stop => actions.push(SongAction::Stop),
                MediaControlEvent::Seek(dir) => actions.push(map_seek(dir, DEFAULT_SEEK_AMOUNT)),
                MediaControlEvent::SeekBy(dir, duration) => actions.push(map_seek(dir, duration.as_secs_f32() * 1000.0)),
                MediaControlEvent::SetPosition(position) => actions.push(SongAction::SetPosition(position.0.as_secs_f32() * 1000.0)),
                MediaControlEvent::SetVolume(vol) => actions.push(SongAction::SetVolume(vol as f32)),
                MediaControlEvent::OpenUri(_) => {},
                #[cfg(feature="graphics")] 
                MediaControlEvent::Raise => actions.push(WindowAction::RequestAttention),
                MediaControlEvent::Quit => {},

                #[cfg(not(feature="graphics"))] _ => {} 
            }
        }
    }
}

fn map_seek(dir: SeekDirection, amount: f32) -> SongAction {
    SongAction::SeekBy(
        amount * match dir {
            SeekDirection::Forward => 1.0,
            SeekDirection::Backward => -1.0,
        }
    )
}


fn map_err(e: souvlaki::Error) -> TatakuError {
    #[cfg(not(windows))] return TatakuError::from_err(e);
    #[cfg(windows)] TatakuError::String(format!("{e:?}"))
}

#[derive(Default2)]
struct LastEventHelper {
    time: TatakuInstant,
    
    #[default(MediaControlEvent::Pause)]
    event: MediaControlEvent,
}
impl LastEventHelper {
    fn new(event: MediaControlEvent) -> Self {
        Self {
            time: TatakuInstant::now(),
            event,
        }
    }
}
