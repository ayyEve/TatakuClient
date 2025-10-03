use crate::*;

/// all time measurements should be in ms
pub trait AudioInstance: Send + Sync {
    fn play(&self, restart: bool);
    fn pause(&self);
    fn stop(&self);
    
    fn get_state(&self) -> AudioState;

    fn get_position(&self) -> f32;
    fn set_position(&self, pos: f32);

    fn set_volume(&self, vol: f32);
    fn set_rate(&self, rate: f32);

    fn set_repeat(&self, repeat: bool);
    fn get_data(&self) -> Vec<tataku::FFTEntry>;
    fn get_duration(&self) -> f32;
}
