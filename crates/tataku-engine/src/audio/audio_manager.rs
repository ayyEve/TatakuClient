use super::audio_api::*;
use crate::prelude::*;

lazy_static::lazy_static!(
    static ref CURRENT_API: Arc<RwLock<Arc<dyn AudioApi>>> = Arc::new(RwLock::new(Arc::new(super::null_audio::NullAudio)));
);


pub struct AudioManager;
impl AudioManager {
    pub fn init_audio(
        engines: Vec<Box<dyn AudioApiInit>>
    ) -> TatakuResult<()> {
        let mut api:Option<Arc<dyn AudioApi>> = None;

        for i in engines {
            match i.init() {
                Ok(good) => { api = Some(good); break; },
                Err(e) => error!("error loading {} api: {e}", i.name())
            }
        }

        // initialize null audio if nothing else works
        if api.is_none() {
            #[cfg(feature = "gameplay")]
            warn!("Audio failed to initialize, using null audio");
            api = Some(Arc::new(super::null_audio::NullAudio));
        }


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