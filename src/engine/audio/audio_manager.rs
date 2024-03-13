use super::audio::*;
use crate::prelude::*;

lazy_static::lazy_static!(
    static ref CURRENT_API: Arc<RwLock<Arc<dyn AudioApi>>> = Arc::new(RwLock::new(Arc::new(super::null_audio::NullAudio)));
);


pub struct AudioManager;
impl AudioManager {
    pub fn init_audio() -> TatakuResult<()> {
        let mut api:Option<Arc<dyn AudioApi>> = None;

        // bass takes priority
        #[cfg(feature = "bass_audio")]
        match super::bass_audio::BassAudio::init() {
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