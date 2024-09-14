use tracing::*;

use std::sync::Arc;
use bass_rs::prelude::*;
use tataku_engine::prelude::{
    AudioApi,
    AudioApiInit,
    TatakuResult,
    AudioError,
    AudioInstance,
    FFTData
};


lazy_static::lazy_static! {
    // wave file bytes with ~1 sample
    // TODO: shouldnt it be possible to make an empty stream directly from bass? should maybe add that to the lib
    static ref EMPTY_STREAM:Arc<StreamChannelInstance> = Arc::new(StreamChannelInstance(StreamChannel::load_from_memory(vec![0x52,0x49,0x46,0x46,0x28,0x00,0x00,0x00,0x57,0x41,0x56,0x45,0x66,0x6D,0x74,0x20,0x10,0x00,0x00,0x00,0x01,0x00,0x02,0x00,0x44,0xAC,0x00,0x00,0x88,0x58,0x01,0x00,0x02,0x00,0x08,0x00,0x64,0x61,0x74,0x61,0x04,0x00,0x00,0x00,0x80,0x80,0x80,0x80], 0i32).expect("error creating empty StreamChannel")));
}


pub struct BassAudio(bass_rs::Bass);
impl AudioApi for BassAudio {
    fn load_sample_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        let channel = SampleChannel::load_from_memory(data, 0i32, 64).map_err(map_bass_err)?;
        Ok(Arc::new(SampleChannelInstance::new(channel)))
    }
    fn load_stream_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        let channel = StreamChannel::load_from_memory(data, 0i32).map_err(map_bass_err)?;
        Ok(Arc::new(StreamChannelInstance(channel)))
    }

    fn empty_audio(&self) -> Arc<dyn AudioInstance> {
        EMPTY_STREAM.clone()
    }

    fn amplitude_multiplier(&self) -> f32 {
        1000.0
    }
}

pub struct BassAudioInit;
impl AudioApiInit for BassAudioInit {
    fn name(&self) -> &'static str { "Bass Audio" }
    fn init(&self) -> TatakuResult<Arc<dyn AudioApi>> {
        Ok(Arc::new(BassAudio(bass_rs::Bass::init_default().map_err(map_bass_err)?)))
    }
}

struct SampleChannelData {
    channel: SampleChannel,
    volume: f32,
    rate: f32,
}
impl SampleChannelData {
    fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
        // for i in self.channel.get_channels() {
        //     let _ = i.set_rate(rate);
        // }
    }
    fn set_vol(&mut self, vol: f32) {
        self.volume = vol;
        // for i in self.channel.get_channels() {
        //     if let Err(e) = i.set_volume(vol) {
        //         warn!("couldnt set vol: {e:?}")
        //     }
        // }
    }
}

pub struct SampleChannelInstance(parking_lot::RwLock<SampleChannelData>);
impl SampleChannelInstance {
    fn new(channel: SampleChannel) -> Self {
        Self(parking_lot::RwLock::new(SampleChannelData { channel, volume: 1.0, rate: 1.0 }))
    }
    fn data(&self) -> parking_lot::RwLockReadGuard<SampleChannelData> {
        self.0.read()
    }
    fn data_mut(&self) -> parking_lot::RwLockWriteGuard<SampleChannelData> {
        self.0.write()
    }
}
impl AudioInstance for SampleChannelInstance {
    fn set_rate(&self, rate: f32) {
        self.data_mut().set_rate(rate);
    }
    fn play(&self, restart: bool) {
        let mut data = self.data_mut();

        let Ok(new_channel) = data.channel.get_channel() else { warn!("couldnt get new channel"); return };
        // make sure the new channel has the correct volume and rate set
        let _ = new_channel.set_rate(data.rate);
        let _ = new_channel.set_volume(data.volume);
        let _ = new_channel.play(restart);
    }

    fn pause(&self) {
        let _ = self.data().channel.pause();
    }

    fn stop(&self) {
        let _ = self.data().channel.stop();
    }

    fn is_playing(&self) -> bool {
        self.data().channel.get_playback_state() == Ok(PlaybackState::Playing)
    }

    fn is_paused(&self) -> bool {
        let state = self.data().channel.get_playback_state();
        state == Ok(PlaybackState::Paused) || state == Ok(PlaybackState::PausedDevice)
    }

    fn is_stopped(&self) -> bool {
        let state = self.data().channel.get_playback_state();
        state == Ok(PlaybackState::Stopped) || state == Ok(PlaybackState::Stalled)
    }

    fn get_position(&self) -> f32 {
        self.data().channel.get_position().unwrap_or_default() as f32
    }

    fn set_position(&self, pos: f32) {
        let _ = self.data().channel.set_position(pos as f64);
    }

    fn set_volume(&self, vol: f32) {
        self.data_mut().set_vol(vol);
    }

    fn get_data(&self) -> Vec<FFTData> {
        self.data().channel
        .get_data(DataType::FFT2048, 1024u32).unwrap_or_default()
        .into_iter()
        .map(FFTData::AmplitudeOnly)
        .collect()
    }

    fn get_duration(&self) -> f32 {
        self.data().channel.get_length_seconds().unwrap_or_default() as f32 * 1000.0
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
        .map(FFTData::AmplitudeOnly)
        .collect()
    }

    fn get_duration(&self) -> f32 {
        self.0.get_length_seconds().unwrap_or_default() as f32 * 1000.0
    }
}




fn map_bass_err(e: BassError) -> AudioError {
    if [BassError::Empty, BassError::Fileform, BassError::Illparam].contains(&e) {
        AudioError::Empty
    } else {
        AudioError::ApiError(format!("{e:?}"))
    }
}

