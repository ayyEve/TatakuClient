use crate::prelude::*;

#[derive(Default)]
pub struct SoundManager {
    sounds: HashMap<String, SoundEntry>,
}
impl SoundManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn handle_action(
        &mut self, 
        action: AudioAction,
        values: &mut ValueCollection,
        engine: &mut AudioManager,
   ) {
        let id = &action.id;
        
        match action.action {
            AudioActionType::Play { volume, repeat:_, restart } => {
                let Some(sound) = self.sounds.get(id) else { return error!("sound not loaded: {id}") };
                let volume = values.settings.get_effect_vol() * volume;
                sound.set_volume(volume);
                sound.play(restart);
            }
            AudioActionType::Stop => {
                let Some(sound) = self.sounds.get(id) else { return error!("sound not loaded: {id}") };
                sound.stop();
            }

            AudioActionType::Load { list } => {
                for i in list {
                    let path = match i.source {
                        HitsoundSource::Default => format!("resources/audio/{}", i.path),
                        // HitsoundSource::Skin => values.skin.current_beatmap.as_ref().unwrap().get_parent_dir().unwrap().join(&i.path).to_string_lossy().to_string(),
                        HitsoundSource::Beatmap if values.beatmap_manager.current_beatmap.is_some() => values.beatmap_manager.current_beatmap.as_ref().unwrap().get_parent_dir().unwrap().join(&i.path).to_string_lossy().to_string(),
                        _ => continue, // FIXME: need a way to get the current skin path easily
                    };

                    let Ok(sound) = engine.load(&path) else { 
                        // error!("sound not found {path}"); 
                        continue 
                    };
                    self.sounds.insert(action.id, SoundEntry { sound, source: i.source });
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
    source: HitsoundSource,
}
impl Deref for SoundEntry {
    type Target = Arc<dyn AudioInstance>;
    fn deref(&self) -> &Self::Target {
        &self.sound
    }
}