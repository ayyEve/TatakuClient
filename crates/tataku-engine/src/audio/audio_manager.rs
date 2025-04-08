use super::audio_api::*;
use crate::prelude::*;

pub struct AudioManager {
    engine: Arc<dyn AudioApi>,
    _engine_builders: Vec<Box<dyn AudioApiInit>>,
}
impl AudioManager {
    pub async fn init_audio(
        engines: Vec<Box<dyn AudioApiInit>>
    ) -> TatakuResult<Self> {
        let mut api: Option<Arc<dyn AudioApi>> = None;

        for i in &engines {
            match i.init().await {
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
            Ok(Self {
                engine: api,
                _engine_builders: engines
            })
        } else {
            Err(TatakuError::String("Failed to load audio api".to_owned()))
        }
        
    }

    // pub fn empty_stream() -> Arc<dyn AudioInstance> { CURRENT_API.read().empty_audio() }
    pub fn amplitude_multiplier(&self) -> f32 { self.engine.amplitude_multiplier() }


    pub fn load_song(&self, path: impl AsRef<Path>) -> TatakuResult<Arc<dyn AudioInstance>> {
        self.engine.load_stream_path(path.as_ref())
    }
    pub fn load_song_raw(&self, bytes: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        self.engine.load_stream_data(bytes)
    }
    
    pub fn load(&self, path: impl AsRef<str>) -> TatakuResult<Arc<dyn AudioInstance>> {
        let path = path.as_ref();
        for ext in [".wav", ".mp3", ".ogg"] {
            let path = format!("{path}{ext}");
            if let Ok(sound) = self.engine.load_sample_path(&path) {
                return Ok(sound)
            }
            // error!("not found: {path}");
        }
        Err(TatakuError::Audio(AudioError::FileDoesntExist))
    }

}
