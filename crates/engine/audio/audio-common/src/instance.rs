use crate::prelude::*;

/// all time measurements should be in ms
pub trait AudioInstance: Send + Sync {
    fn play(&self, restart: bool);
    fn pause(&self);
    fn stop(&self);


    fn is_playing(&self) -> bool;
    fn is_paused(&self) -> bool;
    fn is_stopped(&self) -> bool;


    fn get_position(&self) -> f32;
    fn set_position(&self, pos: f32);

    fn set_volume(&self, vol: f32);
    fn set_rate(&self, rate: f32);

    fn set_repeat(&self, repeat: bool);
    fn get_data(&self) -> Vec<FFTEntry>;
    fn get_duration(&self) -> f32;

    fn get_state(&self) -> AudioState {
        if self.is_playing() {
            AudioState::Playing
        } else if self.is_paused() {
            AudioState::Paused
        } else if self.is_stopped() {
            AudioState::Stopped
        } else {
            AudioState::Unknown
        }
    }
}
