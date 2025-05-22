use crate::prelude::*;

#[derive(Default)]
pub struct SoundManager {
    sounds: HashMap<String, SoundEntry>,
}
impl SoundManager {
    pub fn handle_action(
        &mut self, 
        action: AudioAction,
        values: &mut ValueCollection,
        engine: &mut AudioManager,
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
                                .beatmap_manager.current_beatmap 
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

                    let Ok(sound) = engine.load(&path)
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
