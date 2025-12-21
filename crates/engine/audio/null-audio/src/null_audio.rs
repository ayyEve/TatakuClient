use tataku_engine::*;
use tataku_audio::*;

pub struct NullAudio;
impl AudioApi for NullAudio {
    fn load_sample_data(&self, _: Vec<u8>) -> tataku::Result<Arc<dyn AudioInstance>> {
        Ok(Arc::new(NullAudioInstance))
    }

    fn load_stream_data(&self, _: Vec<u8>) -> tataku::Result<Arc<dyn AudioInstance>> {
        Ok(Arc::new(NullAudioInstance))
    }
}

pub struct NullAudioInstance;
impl AudioInstance for NullAudioInstance {
    fn play(&self, _: bool) {}
    fn pause(&self) {}
    fn stop(&self) {}

    fn get_position(&self) -> f32 { 0.0 }
    fn get_duration(&self) -> f32 { 1.0 }

    fn set_rate(&self, _: f32) {}
    fn set_volume(&self, _: f32) {}
    fn set_position(&self, _: f32) {}
    fn set_repeat(&self, _: bool) {}

    fn get_data(&self) -> Vec<tataku::FFTEntry> { vec![] }
    fn get_state(&self) -> AudioState { AudioState::Stopped }
}
