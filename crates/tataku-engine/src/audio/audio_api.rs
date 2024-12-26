use crate::prelude::*;

pub trait AudioApi: Send + Sync {
    fn load_sample_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>>;
    fn load_stream_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>>;

    fn load_sample_path(&self, path: &Path) -> TatakuResult<Arc<dyn AudioInstance>> {
        let data = Io::read_file(path)?;
        self.load_sample_data(data)
    }
    fn load_stream_path(&self, path: &Path) -> TatakuResult<Arc<dyn AudioInstance>> {
        let data = Io::read_file(path)?;
        self.load_stream_data(data)
    }

    fn empty_audio(&self) -> Arc<dyn AudioInstance>;
    fn amplitude_multiplier(&self) -> f32 { 1.0 }
}

pub trait AudioApiInit: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&self) -> TatakuResult<Arc<dyn AudioApi>>;
}


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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
#[derive(Reflect)]
pub enum AudioState {
    Playing,
    Paused,
    Stopped,
    #[default]
    Unknown,
}


#[derive(Copy, Clone, Debug)]
pub enum FFTEntry {
    AmplitudeOnly(f32),
    AmplitudeAndFrequency(f32, f32)
}
impl FFTEntry {
    pub fn amplitude(self) -> f32 {
        match self {
            FFTEntry::AmplitudeOnly(a) => a,
            FFTEntry::AmplitudeAndFrequency(_, a) => a,
        }
    }
    pub fn set_amplitude(&mut self, na: f32) {
        match self {
            FFTEntry::AmplitudeOnly(a) => *a = na,
            FFTEntry::AmplitudeAndFrequency(_, a) => *a = na,
        }
    }
}
impl From<f32> for FFTEntry {
    fn from(a: f32) -> Self {
        Self::AmplitudeOnly(a)
    }
}
impl From<(f32, f32)> for FFTEntry {
    fn from((f, a): (f32, f32)) -> Self {
        Self::AmplitudeAndFrequency(f, a)
    }
}
impl Default for FFTEntry {
    fn default() -> Self {
        Self::AmplitudeAndFrequency(0.0, 0.0)
    }
}