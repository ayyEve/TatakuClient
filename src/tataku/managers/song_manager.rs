use crate::prelude::*;


pub struct SongManager {
    current_song: Option<SongData>
}
impl SongManager {
    pub fn new() -> Self {
        Self {
            current_song: None,
        }
    }

    pub fn handle_song_set_action(&mut self, action: SongMenuSetAction) -> TatakuResult {
        trace!("Set song: {action:?}");

        match action {
            SongMenuSetAction::None => {
                if let Some(song) = self.current_song.take() {
                    song.instance.stop();
                }
            }

            SongMenuSetAction::FromFile(path) => {
                // make sure the file exists
                if !Io::exists(&path) {
                    error!("Song file does not exist! {path}");
                    return TatakuResult::Err(TatakuError::Audio(AudioError::FileDoesntExist))
                }

                // check if the file is the same as current
                if self.current_song.as_ref().filter(|s| s.id == path).is_some() {
                    trace!("Trying to set the same song as current, ignoring");
                    return Ok(());
                }

                // stop the current audio
                if let Some(song) = &self.current_song {
                    song.instance.stop();
                }
                
                // load the provided audio
                let sound = AudioManager::load_song(&path)?;

                // set out current song to it
                self.current_song = Some(SongData::new(sound, path));

                // play should be handled separately
            }
            
        }

        Ok(())
    }


    pub fn position(&self) -> f32 {
        let Some(song) = &self.current_song else { return 0.0 };
        song.instance.get_position()
    }

    pub fn state(&self) -> AudioState {
        let Some(song) = &self.current_song else { return AudioState::Stopped };
        song.instance.get_state()
    }

    pub fn instance(&self) -> Option<Arc<dyn AudioInstance>> {
        self.current_song.as_ref().map(|c|c.instance.clone())
    }
}

struct SongData {
    instance: Arc<dyn AudioInstance>,
    id: String,
}
impl SongData {
    fn new(instance: Arc<dyn AudioInstance>, path: String) -> Self {
        Self {
            instance,
            id: path
        }
    }
}