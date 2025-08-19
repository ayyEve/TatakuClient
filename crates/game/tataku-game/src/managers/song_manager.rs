use crate::prelude::*;
use tataku_audio::prelude::*;

#[derive(Default)]
pub struct SongManager {
    song_queue: Vec<SongData>,
    current_song: Option<SongData>,

    #[cfg(feature="graphics")] 
    fft_hooks: Vec<Weak<FFTHook>>,
}
impl SongManager {
    fn play_song(
        &mut self, 
        key: ArcStr, 
        mut params: SongPlayData, 
        load_song: impl FnOnce(&mut AudioManager) -> TatakuResult<Arc<dyn AudioInstance>>,
        actions: &mut ActionQueue,
        engine: &mut AudioManager,
        settings: &Settings,
    ) -> TatakuResult<()> {
        // check if the key is the same as current
        if let Some(song) = self.current_song
            .as_ref()
            .filter(|s| s.id == key)
        {
            trace!("Trying to set the same song as current");
            if params.restart {
                params.play = true;
                Self::apply_params(&song.instance, params, settings);
            }

            actions.push(GameAction::HandleEvent(TatakuEventType::SongStart, None));
            return Ok(());
        }

        // try to load the provided audio
        let song = load_song(engine)?;

        // stop the current audio
        if let Some(s) = self.current_song.as_ref() { 
            s.instance.stop();
        }

        // apply params
        Self::apply_params(&song, params, settings);

        // set our current song to the loaded audio
        self.current_song = Some(SongData::new(song, key));

        actions.push(GameAction::HandleEvent(TatakuEventType::SongStart, None));
        Ok(())
    }

    #[cfg(feature="graphics")] 
    fn update_ffts(&mut self, engine: &mut AudioManager) {
        if self.fft_hooks.is_empty() { return }
        let Some(song) = &self.current_song else { return };
        let amp_mult = engine.amplitude_multiplier();

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

    pub fn update(&mut self, engine: &mut AudioManager) {
        #[cfg(feature="graphics")] 
        self.update_ffts(engine);
    }

    pub fn handle_song_set_action(
        &mut self, 
        action: SongSetAction,
        actions: &mut ActionQueue,
        engine: &mut AudioManager,
        settings: &Settings,
    ) -> TatakuResult {
        trace!("Set song: {action:?}");

        match action {
            SongSetAction::Remove => {
                if let Some(song) = self.current_song.take() {
                    song.instance.stop();
                }
            }

            SongSetAction::PushQueue => {
                if let Some(song) = self.current_song.take() {
                    song.instance.pause();
                    self.song_queue.push(song);
                }
            }

            SongSetAction::PopQueue(params) => {
                let Some(popped) = self.song_queue.pop() 
                else { return Ok(()) };

                if let Some(song) = self.current_song.take() {
                    song.instance.stop();
                }

                Self::apply_params(&popped.instance, params, settings);
                self.current_song = Some(popped);
            }

            SongSetAction::FromFile(
                path, 
                params
            ) => self.play_song(
                path.clone(), 
                params, 
                move |engine| engine.load_song(&*path),
                actions,
                engine,
                settings,
            )?,
            
            SongSetAction::FromData(
                data, 
                key, 
                params
            ) => self.play_song(
                key, 
                params, 
                move |engine| engine.load_song_raw(data),
                actions,
                engine,
                settings,
            )?,
        }

        Ok(())
    }

    fn apply_params(
        song: &Arc<dyn AudioInstance>, 
        params: SongPlayData, 
        settings: &Settings
    ) {
        trace!("Using params: {params:?}");
        if params.play { song.play(params.restart) }
        if let Some(pos) = params.position { song.set_position(pos) }
        if let Some(rate) = params.rate { song.set_rate(rate) }
        if let Some(vol) = params.volume { 
            song.set_volume(vol);
        } else {
            song.set_volume(settings.get_music_vol());
        }
    }

    #[cfg(feature="graphics")] 
    pub fn hook_fft(&mut self, hook: Weak<FFTHook>) {
        self.fft_hooks.push(hook);
    }

    pub fn position(&self) -> f32 {
        let Some(song) = &self.current_song 
        else { return 0.0 };

        song.instance.get_position()
    }

    pub fn state(&self) -> AudioState {
        let Some(song) = &self.current_song 
        else { return AudioState::Stopped };

        song.instance.get_state()
    }

    pub fn instance(&self) -> Option<Arc<dyn AudioInstance>> {
        self.current_song.as_ref().map(|c| c.instance.clone())
    }
}

struct SongData {
    instance: Arc<dyn AudioInstance>,
    id: ArcStr,
}
impl SongData {
    fn new(instance: Arc<dyn AudioInstance>, path: ArcStr) -> Self {
        Self {
            instance,
            id: path
        }
    }
}
