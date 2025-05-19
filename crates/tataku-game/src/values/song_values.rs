use crate::prelude::*;

#[derive(Reflect)]
#[derive(Default, Debug, Copy, Clone)]
#[reflect(display = "debug")]
pub struct SongInfo {
    pub position: f32,
    pub paused: bool,
    pub playing: bool,
    pub stopped: bool,
    pub exists: bool,

    pub state: AudioState,
}
impl SongInfo {
    pub fn update(&mut self, audio: Option<Arc<dyn AudioInstance>>) {
        if let Some(audio) = audio {
            self.position = audio.get_position();
            self.set_state(audio.get_state());
            self.exists = true;
        } else {
            self.position = 0.0;
            self.set_state(AudioState::Unknown);
            self.exists = false;
        }
    }
    pub fn set_state(&mut self, state: AudioState) -> bool {
        if self.state == state { return false }

        self.paused = state == AudioState::Paused;
        self.playing = state == AudioState::Playing;
        self.stopped = state == AudioState::Stopped;
        self.exists = self.paused || self.playing || self.stopped;

        self.state = state;

        true
    }
}
