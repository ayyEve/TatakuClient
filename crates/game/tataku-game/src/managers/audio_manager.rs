use crate::prelude::*;
use tataku_audio::prelude::*;

pub struct AudioManager {
    engine: Arc<dyn AudioApi>,
    sounds: HashMap<String, SoundEntry>,

    _engine_builders: Vec<AudioApiInit>,
}
impl AudioManager {
    pub fn init_audio(
        engines: Vec<AudioApiInit>
    ) -> TatakuResult<Self> {
        let mut api: Option<Arc<dyn AudioApi>> = None;

        for i in &engines {
            match (i.init)() {
                Ok(good) => { api = Some(good); break; },
                Err(e) => error!("error loading {} api: {e}", i.name)
            }
        }

        // initialize null audio if nothing else works
        if api.is_none() {
            #[cfg(feature = "gameplay")]
            warn!("Audio failed to initialize, using null audio");
            api = Some(Arc::new(tataku_null_audio::NullAudio));
        }

        if let Some(api) = api {
            Ok(Self {
                engine: api,
                sounds: HashMap::new(),
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


        pub fn handle_action(
            &mut self, 
            action: AudioAction,
            values: &mut ValueCollection,
            #[cfg(feature="graphics")] 
            skin: &mut SkinManager,
        ) {
        let id = &action.id;
        
        match action.action {
            AudioActionType::Play { 
                volume, 
                repeat, 
                restart 
            } => {
                let Some(sound) = self.sounds.get(id) else { 
                    return error!("sound not loaded: {id}");
                };
                let volume = values.settings.get_effect_vol() * volume;
                sound.set_volume(volume);
                sound.set_repeat(repeat);
                sound.play(restart);
            }
            AudioActionType::Stop => {
                let Some(sound) = self.sounds.get(id) else { 
                    return error!("sound not loaded: {id}");
                };
                sound.stop();
            }

            AudioActionType::Load { list } => {
                // debug!("loading sounds: {list:?}");
                for i in list {
                    let path = match i.source {
                        HitsoundSource::Default => format!("resources/audio/{}", i.path),
                        
                        HitsoundSource::Beatmap => {
                            let Some(map) = &values
                                .beatmap_manager.current_beatmap()
                            else {
                                continue 
                            };

                            map
                            .get_parent_dir()
                            .unwrap()
                            .join(&i.path)
                            .to_string_lossy()
                            .to_string()
                        }

                        #[cfg(feature="graphics")] 
                        HitsoundSource::Skin => skin
                            .skin_path()
                            .parent()
                            .unwrap()
                            .join(&i.path)
                            .to_string_lossy()
                            .to_string(),
                        
                        #[cfg(not(feature="graphics"))] 
                        _ => continue
                    };

                    let Ok(sound) = self.load(&path)
                    else { 
                        // error!("sound not found {path}"); 
                        continue 
                    };

                    self.sounds.insert(
                        action.id, 
                        SoundEntry { sound, source: i.source }
                    );
                    break;
                }
            }

            AudioActionType::Unload => {
                self.sounds.remove(id);
            }
        }
    }

}



struct SoundEntry {
    sound: Arc<dyn AudioInstance>,
    #[allow(unused, reason = "will be used later")]
    source: HitsoundSource,
}
impl Deref for SoundEntry {
    type Target = Arc<dyn AudioInstance>;
    fn deref(&self) -> &Self::Target {
        &self.sound
    }
}

