use crate::prelude::*;


pub struct SongManager {
    song_queue: Vec<SongData>,
    current_song: Option<SongData>
}
impl SongManager {
    pub fn new() -> Self {
        Self {
            song_queue: Vec::new(),
            current_song: None,
        }
    }

    pub fn handle_song_set_action(&mut self, action: SongMenuSetAction) -> TatakuResult {
        trace!("Set song: {action:?}");

        match action {
            SongMenuSetAction::Remove => {
                if let Some(song) = self.current_song.take() {
                    song.instance.stop();
                }
            }

            SongMenuSetAction::PushQueue => {
                if let Some(song) = self.current_song.take() {
                    song.instance.pause();
                    self.song_queue.push(song);
                }
            }

            SongMenuSetAction::PopQueue => {
                if let Some(song) = self.current_song.take() {
                    song.instance.stop();
                }

                self.current_song = self.song_queue.pop();
                if let Some(song) = &self.current_song {
                    song.instance.play(false);
                }
            }


            SongMenuSetAction::FromFile(path, mut params) => {
                // make sure the file exists
                if !Io::exists(&path) {
                    error!("Song file does not exist! {path}");
                    return TatakuResult::Err(TatakuError::Audio(AudioError::FileDoesntExist))
                }

                // check if the file is the same as current
                if let Some(song) = self.current_song.as_ref().filter(|s|s.id == path) {
                    debug!("Trying to set the same song as current");
                    if params.restart {
                        params.play = true;
                        Self::apply_params(&song.instance, params);
                    }

                    return Ok(());
                }

                // try to load the provided audio
                let song = AudioManager::load_song(&path)?;

                // stop the current audio
                self.current_song.ok_do(|s| s.instance.stop());
                
                // apply params
                Self::apply_params(&song, params);

                // set our current song to the loaded audio
                self.current_song = Some(SongData::new(song, path));
            }

            SongMenuSetAction::FromData(data, key, mut params) => {
                // check if the key is the same as current
                if let Some(song) = self.current_song.as_ref().filter(|s|s.id == key) {
                    debug!("Trying to set the same song as current");
                    if params.restart {
                        params.play = true;
                        Self::apply_params(&song.instance, params);
                    }

                    return Ok(());
                }

                // try to load the provided audio
                let song = AudioManager::load_song_raw(data)?;

                // stop the current audio
                self.current_song.ok_do(|s| s.instance.stop());

                // apply params
                Self::apply_params(&song, params);
                
                // set our current song to the loaded audio
                self.current_song = Some(SongData::new(song, key));
            }
        }

        Ok(())
    }

    fn apply_params(song: &Arc<dyn AudioInstance>, params: SongPlayData) {
        info!("Using params: {params:?}");
        if params.play { song.play(params.restart) }
        params.position.map(|pos| song.set_position(pos));
        params.rate.map(|rate| song.set_rate(rate));
        params.volume.map(|vol| song.set_volume(vol));
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
        self.current_song.as_ref().map(|c| c.instance.clone())
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

#[derive(Copy, Clone, Debug, Default)]
pub struct SongPlayData {
    pub play: bool,
    pub restart: bool,
    pub position: Option<f32>,
    pub rate: Option<f32>,
    pub volume: Option<f32>,
}
