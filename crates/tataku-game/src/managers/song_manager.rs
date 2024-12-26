use crate::prelude::*;

#[derive(Default)]
pub struct SongManager {
    song_queue: Vec<SongData>,
    current_song: Option<SongData>,

    fft_hooks: Vec<Weak<FFTHook>>,
}
impl SongManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn play_song(
        &mut self, 
        key: String, 
        mut params: SongPlayData, 
        load_song: impl FnOnce() -> TatakuResult<Arc<dyn AudioInstance>>
    ) -> TatakuResult<()> {
        // check if the key is the same as current
        if let Some(song) = self.current_song.as_ref().filter(|s| s.id == key) {
            trace!("Trying to set the same song as current");
            if params.restart {
                params.play = true;
                Self::apply_params(&song.instance, params);
            }

            return Ok(());
        }

        // try to load the provided audio
        let song = load_song()?;
        // let song = AudioManager::load_song_raw(data)?;

        // stop the current audio
        if let Some(s) = self.current_song.as_ref() { 
            s.instance.stop() 
        }

        // apply params
        Self::apply_params(&song, params);

        // set our current song to the loaded audio
        self.current_song = Some(SongData::new(song, key));

        Ok(())
    }

    fn update_ffts(&mut self) {
        if self.fft_hooks.is_empty() { return }
        let Some(song) = &self.current_song else { return };
        let amp_mult = AudioManager::amplitude_multiplier();

        let data = song.instance.get_data();
        self.fft_hooks.retain(|h| {
            let Some(hook) = h.upgrade() else { return false };
            if let Some(mut a) = hook.try_write() {
                a.amplitude_multiplier = amp_mult;
                a.data = data.clone();
            }

            true
        });
    }

    pub fn update(&mut self) {
        self.update_ffts();
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

            SongMenuSetAction::FromFile(path, params) => self.play_song(
                path.clone(), 
                params, 
                move || AudioManager::load_song(&path)
            )?,
            
            SongMenuSetAction::FromData(data, key, params) => self.play_song(
                key, 
                params, 
                move || AudioManager::load_song_raw(data)
            )?,
        }

        Ok(())
    }

    fn apply_params(song: &Arc<dyn AudioInstance>, params: SongPlayData) {
        trace!("Using params: {params:?}");
        if params.play { song.play(params.restart) }
        if let Some(pos) = params.position { song.set_position(pos) }
        if let Some(rate) = params.rate { song.set_rate(rate) }
        if let Some(vol) = params.volume { song.set_volume(vol) }
    }

    pub fn hook_fft(&mut self, hook: Weak<FFTHook>) {
        self.fft_hooks.push(hook);
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
