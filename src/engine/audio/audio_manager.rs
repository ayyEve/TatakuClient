use super::audio::*;
use crate::prelude::*;

lazy_static::lazy_static!(
    static ref CURRENT_SONG: Arc<Mutex<Option<(String, Arc<dyn AudioInstance>)>>> = Arc::new(Mutex::new(None));

    static ref PLAY_PENDING: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    static ref CURRENT_API: Arc<parking_lot::RwLock<Arc<dyn AudioApi>>> = Arc::new(parking_lot::RwLock::new(Arc::new(super::null_audio::NullAudio)));
);


pub struct AudioManager;
impl AudioManager {
    pub fn init_audio(window_ptr: *mut std::ffi::c_void) -> TatakuResult<()> {
        let mut api:Option<Arc<dyn AudioApi>> = None;

        // bass takes priority
        #[cfg(feature = "bass_audio")]
        match super::bass_audio::BassAudio::init(window_ptr) {
            Ok(bass) => api = Some(Arc::new(bass)),
            Err(e) => error!("error loading bass api: {e}"),
        }

        // if it failed to load, or is disabled, try kira
        // #[cfg(feature = "kira_audio")]
        // if api.is_none() {
        //     match super::kira_audio::KiraAudio::init(window_ptr) {
        //         Ok(kira) => api = Some(Arc::new(kira)),
        //         Err(e) => error!("error loading kira api: {e}"),
        //     }
        // }


        if let Some(api) = api {
            *CURRENT_API.write() = api;
            Ok(())
        } else {
            Err(TatakuError::String("Failed to load audio api".to_owned()))
        }
        
    }

    pub fn empty_stream() -> Arc<dyn AudioInstance> { CURRENT_API.read().empty_audio() }
    pub fn amplitude_multiplier() -> f32 { CURRENT_API.read().amplitude_multiplier() }

    pub async fn play_song(path: impl AsRef<str>, restart:bool, position: f32) -> TatakuResult<Arc<dyn AudioInstance>> {
        trace!("play_song - playing {}", path.as_ref());
        // check if we're already playing, if restarting is allowed
        let string_path = path.as_ref().to_owned();

        // check if file exists
        {
            if !exists(&string_path) {
                error!("audio file does not exist! {}", string_path);
                return TatakuResult::Err(TatakuError::Audio(AudioError::FileDoesntExist))
            }
        }

        if let Some((c_path, audio)) = CURRENT_SONG.lock().await.clone() {
            if c_path != string_path {
                trace!("play_song - pre-stopping old song");
                audio.stop();
            }
        }
        
        
        // let id = format!("{}", uuid::Uuid::new_v4());

        // // set the pending song to us
        // *PLAY_PENDING.lock() = id.clone();

        // // // load the audio data (this is what takes a million years)
        // // let sound = Audio::load_song(path.as_ref())?;

        // // if the pending song is no longer us
        // if *PLAY_PENDING.lock() != id {
        //     trace!("play_song - pending song changed, leaving");
        //     return Err(AudioError::DifferentSong.into())
        // }

        match CURRENT_SONG.lock().await.clone() {
            Some((c_path, audio)) => { // audio set
                if string_path == c_path { // same file as what we want to play
                    if restart {
                        trace!("play_song - same song, restarting"); 
                        audio.set_position(position);
                    }
                    trace!("play_song - same song, exiting");
                    return Ok(audio);
                } else { // different audio
                    trace!("play_song - stopping old song");
                    audio.stop();
                }
            }
            None => trace!("play_song - no audio"), // no audio set
        }

        let sound = AudioManager::load_song(path.as_ref())?;

        // double check the song is stopped when we get here
        if let Some((_, song)) = CURRENT_SONG.lock().await.clone() {
            if song.is_playing() {
                // trace!("double stopping song: {}", Arc::strong_count(&song.channel.handle));
                song.stop();
            }
        }

        sound.play(true);
        sound.set_position(position);
        sound.set_volume(get_settings!().get_music_vol());

        *CURRENT_SONG.lock().await = Some((string_path, sound.clone()));
        Ok(sound)
    }
    
    pub async fn play_song_raw(key: impl AsRef<str>, bytes: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        // stop current
        AudioManager::stop_song().await;

        let sound = AudioManager::load_song_raw(bytes)?;
        sound.play(true);
        sound.set_volume(get_settings!().get_music_vol());
        
        *CURRENT_SONG.lock().await = Some((key.as_ref().to_owned(), sound.clone()));
        Ok(sound)
    }
    
    pub async fn stop_song() {
        trace!("stopping song");
        if let Some(audio) = AudioManager::get_song().await {
            audio.stop();
        }

        *CURRENT_SONG.lock().await = None;
    }
    pub async fn get_song() -> Option<Arc<dyn AudioInstance>> {
        if let Some((_, audio)) = CURRENT_SONG.lock().await.clone() {
            return Some(audio)
        }
        None
    }
    pub async fn get_song_raw() -> Option<(String, Arc<dyn AudioInstance>)> {
        CURRENT_SONG.lock().await.clone()
    }

    pub fn load_song(path: impl AsRef<Path>) -> TatakuResult<Arc<dyn AudioInstance>> {
        CURRENT_API.read().load_stream_path(path.as_ref())
    }
    pub fn load_song_raw(bytes: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        CURRENT_API.read().load_stream_data(bytes)
    }
    
    pub fn load(path: impl AsRef<Path>) -> TatakuResult<Arc<dyn AudioInstance>> {
        CURRENT_API.read().load_sample_path(path.as_ref())
    }
}