use super::AudioApi;
use crate::prelude::*;

pub struct NullAudio;
impl AudioApi for NullAudio {
    fn init() -> TatakuResult<Self> where Self:Sized {
        Ok(Self)
    }

    fn load_sample_data(&self, _: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        Ok(Arc::new(NullAudioInstance))
    }

    fn load_stream_data(&self, _: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        Ok(Arc::new(NullAudioInstance))
    }

    fn empty_audio(&self) -> Arc<dyn AudioInstance> {
        Arc::new(NullAudioInstance)
    }
}

pub struct NullAudioInstance;
impl AudioInstance for NullAudioInstance {
    fn play(&self, _: bool) {}
    fn pause(&self) {}
    fn stop(&self) {}

    fn is_playing(&self) -> bool { false }
    fn is_paused(&self) -> bool { false }
    fn is_stopped(&self) -> bool { false }

    fn get_position(&self) -> f32 { 0.0 }
    fn get_duration(&self) -> f32 { 1.0 }

    fn set_rate(&self, _: f32) {}
    fn set_volume(&self, _: f32) {}
    fn set_position(&self, _: f32) {}

    fn get_data(&self) -> Vec<FFTData> { vec![] }
}
