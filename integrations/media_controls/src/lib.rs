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
    fn name(&self) -> Cow<'static, str> { "media_controls_integration".into() }
    
    #[allow(unused)]
    fn init(
        &mut self, 
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

        println!("init media controls");
        self.media_controls = Some(MediaControls::new(PlatformConfig {
            dbus_name: "tataku.player",
            display_name: "Tataku!",
            hwnd,
        }).map_err(TatakuError::from_err)?);
        println!("init success");

        Ok(())
    }
    
    fn check_enabled(
        &mut self, 
        settings: &Settings,
    ) -> TatakuResult<()> {
        let Some(controls) = self.media_controls.as_mut() else { return Ok(()) };
        self.enabled = settings.integrations.media_controls;
        println!("enabled: {}", self.enabled);
        println!("attached: {}", self.attached);

        if self.enabled && !self.attached {
            println!("attaching media controls");
            self.attached = true;
            let sender = self.sender.clone();
            controls
                .attach(move |e| sender.send(e).unwrap())
                .map_err(TatakuError::from_err)?;
            println!("media controls attached");
        } else if !self.enabled && self.attached {
            debug!("detaching media controls");
            self.attached = false;
            controls
                .detach()
                .map_err(TatakuError::from_err)?;
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

        match event {
            // TatakuIntegrationEvent::BeatmapStarted { 
            //     beatmap, 
            //     ..
            // } => {
            //     if let Err(e) = controls.set_metadata(souvlaki::MediaMetadata { 
            //         artist: Some(&beatmap.artist), 
            //         title: Some(&beatmap.title), 
            //         album: None, 
            //         cover_url: None, 
            //         duration: Some(Duration::from_secs_f32(*duration / 1000.0))
            //     }) {
            //         error!("error setting metadata: {e:?}")
            //     }
            // }
            // TatakuIntegrationEvent::BeatmapEnded => {
                
            // }
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
                    error!("error setting metadata: {e:?}")
                } else {
                    println!("jbivgnjidmvrnhumijdkbguhnijmkvygbnhjmk");
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
                MediaControlEvent::Raise => actions.push(WindowAction::RequestAttention),
                MediaControlEvent::Quit => {},
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

// enum MediaControlAction {
    
// }


struct LastEventHelper {
    time: TatakuInstant,
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
impl Default for LastEventHelper {
    fn default() -> Self {
        Self {
            time: TatakuInstant::now(),
            event: MediaControlEvent::Pause,
        }
    }
}




// /// helper for interfacing with the os media controls
// pub struct MediaControlsManager {
//     // event_receiver: AsyncUnboundedReceiver<MediaControlEvent>,
//     // internal_event_sender: AsyncUnboundedSender<MediaControlEvent>,

//     // when it was received, the event, was it handled upstream?
//     last_event: LastEventHelper,
//     // event_sender: AsyncUnboundedSender<MediaControlHelperEvent>,
//     last_state: MediaPlaybackState,

//     /// if we should accept control inputs
//     pub enabled: bool,
//     /// if controls are enabled at all
//     controls_enabled: bool,
//     current_metadata: MediaControlMetadata,
// }

// impl MediaControlsManager {
//     pub fn new() -> Self {
//         // let (sender, receiver) = async_unbounded_channel();

//         Self {
//             // event_receiver: receiver,
//             // internal_event_sender: sender,
//             // event_sender,

//             enabled: false,
//             controls_enabled: false,
//             current_metadata: MediaControlMetadata::default(),

//             last_event: LastEventHelper::default(),
//             last_state: MediaPlaybackState::Stopped
//         }
//     }

//     // pub fn update_info(&mut self, map: &Option<Arc<BeatmapMeta>>, duration: f32) {
//     //     self.current_metadata = map.as_ref().map(|map| MediaControlMetadata {
//     //         title: Some(map.title.clone()),
//     //         artist: Some(map.artist.clone()),
//     //         cover_url: Some("file://".to_owned() + Path::new(&map.image_filename).canonicalize().map(|p|p.to_string_lossy().to_string()).unwrap_or_default().trim_start_matches("\\\\?\\")),
//     //         duration: Some(duration),
//     //     }).unwrap_or_default();
        
//     //     Self::set_metadata(&self.current_metadata);
//     // }

//     // #[cfg(feature="graphics")]
//     // fn bind(&self) {
//     //     if !self.controls_enabled { return }
//     //     let controls = GameWindow::get_media_controls();
//     //     let sender = self.internal_event_sender.clone();
//     //     let _ = controls.lock().attach(move |event| sender.send(event).nope() );
//     // }
//     // pub fn set_metadata(&self, meta: &MediaControlMetadata) {
//     //     if !self.controls_enabled { return }
//     //     let meta = meta.clone();

//     //     fn s(a: &Option<String>) -> Option<&str> {
//     //         a.as_ref().map(|x| &**x)
//     //     }

//     //     // duration: Some(Duration::from_millis(duration as u64))
//     //     tokio::task::spawn_blocking(move || {
//     //         let info = MediaMetadata {
//     //             title: s(&meta.title),
//     //             artist: s(&meta.artist),
//     //             album: None,
//     //             cover_url: s(&meta.cover_url).filter(|s|!s.is_empty()),
//     //             duration: meta.duration.map(|d|Duration::from_millis(d as u64))
//     //         };

//     //         #[cfg(feature="graphics")]
//     //         if let Err(e) = GameWindow::get_media_controls().lock().set_metadata(info) {
//     //             warn!("Error setting metadata: {e:?}");
//     //         }
//     //     });
//     // }
//     // pub fn set_playback(&self, state: MediaPlayback) {
//     //     if !self.controls_enabled { return }

//     //     #[cfg(feature="graphics")]
//     //     tokio::task::spawn_blocking(move || {
//     //         if let Err(e) = GameWindow::get_media_controls().lock().set_playback(state) {
//     //             warn!("Error setting playback state: {e:?}");
//     //         }
//     //     });
//     // }

//     pub fn update(
//         &mut self, 
//         settings: &Settings,
//         actions: &mut ActionQueue,
//     ) {
//         if self.controls_enabled != settings.integrations.media_controls {
//             self.controls_enabled = settings.integrations.media_controls;

//             if self.controls_enabled {
//                 actions.push(MediaControlAction::Attach);
//             } else {
//                 actions.push(MediaControlAction::Detatch);
//             }
//         }

//         // if !self.enabled { return }
//     }


//     pub fn handle_event(
//         &mut self, 
//         event: MediaControlEvent, 
//         actions: &mut ActionQueue,
//     ) {
//         if !self.enabled || !self.controls_enabled { return }
//         if event == self.last_event.event || self.last_event.time.as_millis() < MINIMUM_WAIT_BETWEEN_EVENTS { return }

//         match event {
//             MediaControlEvent::Play => actions.push(SongAction::Play),
//             MediaControlEvent::Pause => actions.push(SongAction::Pause),
//             MediaControlEvent::Toggle => actions.push(SongAction::Toggle),
//             MediaControlEvent::Next => actions.push(BeatmapAction::Next),
//             MediaControlEvent::Previous => actions.push(BeatmapAction::Previous(MapActionIfNone::Random(false))),
//             MediaControlEvent::Stop => actions.push(SongAction::Stop),
//             MediaControlEvent::Seek(dir) => actions.push(SongAction::SeekBy(500.0 * from_direction(dir))),
//             MediaControlEvent::SeekBy(dir, amt) => actions.push(SongAction::SeekBy(amt.as_secs_f32() * 1000.0 * from_direction(dir))),
//             MediaControlEvent::SetPosition(pos) => actions.push(SongAction::SetPosition(pos.0.as_secs_f32() * 1000.0)),
//             MediaControlEvent::SetVolume(vol) => actions.push(SongAction::SetVolume(vol as f32)),
//             MediaControlEvent::Quit => actions.push(GameAction::Quit),
            
//             // MediaControlEvent::OpenUri(_url) => {},
//             // MediaControlEvent::Raise => actions.push(WindowAction::BringToFront),
//             _ => {}
//         }

//         self.last_event = LastEventHelper::new(event);
//     }
// }



// impl TatakuIntegration for MediaControlHelper {
//     fn name(&self) -> Cow<'static, str> { "Media Controls".into() }

//     fn init(
//         &mut self, 
//         settings: &Settings
//     ) -> TatakuResult<()> {
//         self.controls_enabled = settings.integrations.media_controls;
//         if self.controls_enabled {
//             self.bind();
//             self.set_metadata(&self.current_metadata);
//         }
        

//         Ok(())
//     }

//     fn check_enabled(
//         &mut self, 
//         settings: &Settings
//     ) -> TatakuResult<()> {
//         if self.controls_enabled != settings.integrations.media_controls {
//             self.controls_enabled = settings.integrations.media_controls;
//             if self.controls_enabled {
//                 self.bind();
//                 self.set_metadata(&self.current_metadata);
//             }
//         }

//         Ok(())
//     }

//     fn handle_event(&mut self, event: &TatakuEvent) {
//         match event {
//             TatakuEvent::BeatmapStarted { start_time, beatmap, .. } => {

//             }
//             TatakuEvent::SongChanged { artist, title, elapsed, duration, image_path } => {
//                 self.current_metadata = MediaControlMetadata {
//                     title: Some(title.clone()),
//                     artist: Some(artist.clone()),
//                     cover_url: Some("file://".to_owned() + Path::new(image_path).canonicalize().map(|p| p.to_string_lossy().to_string()).unwrap_or_default().trim_start_matches("\\\\?\\")),
//                     duration: Some(*duration),
//                 };
            
//                 self.set_metadata(&self.current_metadata);
//             }
            
//             _ => return
//         }
//     }

//     fn update(
//         &mut self, 
//         _values: &mut ValueCollection, 
//         actions: &mut ActionQueue
//     ) {
        

//         // update events 
//         if let Ok(event) = self.event_receiver.try_recv() {
//         }

//         let (last_state, last_elapsed) = match self.last_state {
//             MediaPlaybackState::Playing(e) => (0, e),
//             MediaPlaybackState::Paused(e) => (1, e),
//             MediaPlaybackState::Stopped => (2, 0.0),
//         };


//         let (current_state, current_elapsed) = match song_state {
//             MediaPlaybackState::Playing(e) => (0, e),
//             MediaPlaybackState::Paused(e) => (1, e),
//             MediaPlaybackState::Stopped => (2, 10000.0),
//         };

//         // only update once every Xms
//         if last_state == current_state && current_elapsed - last_elapsed < 1000.0 { return }
//         self.last_state = song_state;

//         // update the playback state
//         Self::set_playback(song_state.into());
//     }
// }



// impl Drop for MediaControlHelper {
//     fn drop(&mut self) {
//         Self::set_metadata(&Default::default());
//         Self::set_playback(MediaPlayback::Stopped);
//     }
// }


// #[derive(Clone, Debug)]
// pub enum MediaControlHelperEvent {
//     Play,
//     Pause,
//     Stop,
//     Toggle,

//     Next,
//     Previous,

//     SeekForward,
//     SeekBackward,
//     SeekForwardBy(f32),
//     SeekBackwardBy(f32),
//     SetPosition(f32),
//     SetVolume(f32),
//     OpenUri(String),
//     Raise,
//     Quit,
// }
// impl From<MediaControlEvent> for MediaControlHelperEvent {
//     fn from(value: MediaControlEvent) -> Self {
//         match value {
//             MediaControlEvent::Play => Self::Play,
//             MediaControlEvent::Pause => Self::Pause,
//             MediaControlEvent::Toggle => Self::Toggle,
//             MediaControlEvent::Next => Self::Next,
//             MediaControlEvent::Previous => Self::Previous,
//             MediaControlEvent::Stop => Self::Stop,
//             MediaControlEvent::SetVolume(vol) => Self::SetVolume(vol as f32),
//             MediaControlEvent::Seek(SeekDirection::Forward) => Self::SeekForward,
//             MediaControlEvent::Seek(SeekDirection::Backward) => Self::SeekBackward,
//             MediaControlEvent::SeekBy(SeekDirection::Forward, d) => Self::SeekForwardBy(d.as_secs_f32() * 1000.0),
//             MediaControlEvent::SeekBy(SeekDirection::Backward, d) => Self::SeekBackwardBy(d.as_secs_f32() * 1000.0),
//             MediaControlEvent::SetPosition(pos) => Self::SetPosition(pos.0.as_secs_f32() * 1000.0),
//             MediaControlEvent::OpenUri(uri) => Self::OpenUri(uri),
//             MediaControlEvent::Raise => Self::Raise,
//             MediaControlEvent::Quit => Self::Quit,
//         }
//     }
// }



// fn from_direction(dir: SeekDirection) -> f32 {
//     match dir {
//         SeekDirection::Backward => -1.0,
//         SeekDirection::Forward => 1.0,
//     }
// }