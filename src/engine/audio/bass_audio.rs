use super::audio::*;
use crate::prelude::*;
use bass_rs::prelude::*;


lazy_static::lazy_static! {
    // wave file bytes with ~1 sample
    // TODO: shouldnt it be possible to make an empty stream directly from bass? should maybe add that to the lib
    static ref EMPTY_STREAM:Arc<StreamChannelInstance> = Arc::new(StreamChannelInstance(StreamChannel::load_from_memory(vec![0x52,0x49,0x46,0x46,0x28,0x00,0x00,0x00,0x57,0x41,0x56,0x45,0x66,0x6D,0x74,0x20,0x10,0x00,0x00,0x00,0x01,0x00,0x02,0x00,0x44,0xAC,0x00,0x00,0x88,0x58,0x01,0x00,0x02,0x00,0x08,0x00,0x64,0x61,0x74,0x61,0x04,0x00,0x00,0x00,0x80,0x80,0x80,0x80], 0i32).expect("error creating empty StreamChannel")));
}


pub struct BassAudio(bass_rs::Bass);
impl AudioApi for BassAudio {
    fn init() -> TatakuResult<Self> where Self:Sized {
        // initialize bass
        Ok(Self(bass_rs::Bass::init_default()?))
    }

    fn load_sample_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        let channel = SampleChannel::load_from_memory(data, 0i32, 32)?;
        Ok(Arc::new(SampleChannelInstance(channel)))
    }
    fn load_stream_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        let channel = StreamChannel::load_from_memory(data, 0i32)?;
        Ok(Arc::new(StreamChannelInstance(channel)))
    }

    fn empty_audio(&self) -> Arc<dyn AudioInstance> {
        EMPTY_STREAM.clone()
    }

    fn amplitude_multiplier(&self) -> f32 {
        1000.0
    }
}

pub struct SampleChannelInstance(SampleChannel);
impl AudioInstance for SampleChannelInstance {
    fn set_rate(&self, rate: f32) {
        let _ = self.0.set_rate(rate);
    }
    fn play(&self, restart: bool) {
        let _ = self.0.play(restart);
    }

    fn pause(&self) {
        let _ = self.0.pause(); 
    }

    fn stop(&self) {
        let _ = self.0.stop(); 
    }

    fn is_playing(&self) -> bool {
        self.0.get_playback_state() == Ok(PlaybackState::Playing)
    }

    fn is_paused(&self) -> bool {
        let state = self.0.get_playback_state();
        state == Ok(PlaybackState::Paused) || state == Ok(PlaybackState::PausedDevice)
    }

    fn is_stopped(&self) -> bool {
        let state = self.0.get_playback_state();
        state == Ok(PlaybackState::Stopped) || state == Ok(PlaybackState::Stalled)
    }

    fn get_position(&self) -> f32 {
        self.0.get_position().unwrap_or_default() as f32
    }

    fn set_position(&self, pos: f32) {
        let _ = self.0.set_position(pos as f64);
    }

    fn set_volume(&self, vol: f32) {
        let _ = self.0.set_volume(vol);
    }

    fn get_data(&self) -> Vec<FFTData> {
        self.0.get_data(DataType::FFT2048, 1024u32).unwrap_or_default()
        .into_iter()
        .map(|a|FFTData::AmplitudeOnly(a))
        .collect()
    }

    fn get_duration(&self) -> f32 {
        self.0.get_length_seconds().unwrap_or_default() as f32 * 1000.0
    }
}

pub struct StreamChannelInstance(StreamChannel);
impl AudioInstance for StreamChannelInstance {
    fn set_rate(&self, rate: f32) {
        let _ = self.0.set_rate(rate);
    }
    fn play(&self, restart: bool) {
        let _ = self.0.play(restart);
    }

    fn pause(&self) {
        let _ = self.0.pause(); 
    }

    fn stop(&self) {
        let _ = self.0.stop(); 
    }

    fn is_playing(&self) -> bool {
        self.0.get_playback_state() == Ok(PlaybackState::Playing)
    }

    fn is_paused(&self) -> bool {
        let state = self.0.get_playback_state();
        state == Ok(PlaybackState::Paused) || state == Ok(PlaybackState::PausedDevice)
    }

    fn is_stopped(&self) -> bool {
        let state = self.0.get_playback_state();
        state == Ok(PlaybackState::Stopped) || state == Ok(PlaybackState::Stalled)
    }

    fn get_position(&self) -> f32 {
        self.0.get_position().unwrap_or_default() as f32
    }

    fn set_position(&self, pos: f32) {
        let _ = self.0.set_position(pos as f64);
    }

    fn set_volume(&self, vol: f32) {
        let _ = self.0.set_volume(vol);
    }

    fn get_data(&self) -> Vec<FFTData> {
        self.0.get_data(DataType::FFT2048, 1024u32).unwrap_or_default()
        .into_iter()
        .map(|a|FFTData::AmplitudeOnly(a))
        .collect()
    }

    fn get_duration(&self) -> f32 {
        self.0.get_length_seconds().unwrap_or_default() as f32 * 1000.0
    }
}


impl From<BassError> for TatakuError {
    fn from(e: BassError) -> Self {
        TatakuError::Audio(e.into())
    }
}

impl From<BassError> for AudioError {
    fn from(e: BassError) -> Self {
        if [BassError::Empty, BassError::Fileform, BassError::Illparam].contains(&e) {
            AudioError::Empty
        } else {
            AudioError::ApiError(format!("{e:?}"))
        }
    }
}
