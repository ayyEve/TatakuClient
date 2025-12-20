use tracing::*;

use std::sync::Arc;
use tataku_audio::*;
use bass_rs::prelude::*;
use tataku_engine_common::errors;
use tataku_engine_common::prelude as tataku;

pub struct BassAudio(bass_rs::Bass);
impl BassAudio {
    fn init() -> tataku::Result<Arc<dyn AudioApi>> {
        check_bass()?;
        Ok(Arc::new(BassAudio(bass_rs::Bass::init_default().map_err(map_bass_err)?)))
    }
}
impl AudioApi for BassAudio {
    fn load_sample_data(&self, data: Vec<u8>) -> tataku::Result<Arc<dyn AudioInstance>> {
        let channel = SampleChannel::load_from_memory(data, 0, 64).map_err(map_bass_err)?;
        Ok(Arc::new(SampleChannelInstance::new(channel)))
    }
    fn load_stream_data(&self, data: Vec<u8>) -> tataku::Result<Arc<dyn AudioInstance>> {
        let channel = StreamChannel::load_from_memory(data, 0).map_err(map_bass_err)?;
        Ok(Arc::new(StreamChannelInstance(channel)))
    }

    fn amplitude_multiplier(&self) -> f32 {
        1000.0
    }
}

#[allow(non_upper_case_globals)]
pub const BassAudioInit: AudioApiInit = AudioApiInit {
    name: "Bass Audio",
    init: BassAudio::init,
};


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
    fn data<'a>(&'a self) -> parking_lot::RwLockReadGuard<'a, SampleChannelData> {
        self.0.read()
    }
    fn data_mut<'a>(&'a self) -> parking_lot::RwLockWriteGuard<'a, SampleChannelData> {
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

    fn get_state(&self) -> AudioState {
        let Ok(state) = self.data().channel.get_playback_state() 
        else { return AudioState::Unknown };
        match state {
            PlaybackState::Playing => AudioState::Playing,
            
            PlaybackState::Paused
            | PlaybackState::PausedDevice => AudioState::Paused,

            PlaybackState::Stopped 
            | PlaybackState::Stalled => AudioState::Stopped,
        }
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
    
    fn set_repeat(&self, _repeat: bool) {
        // let channel = self.data().channel.clone();

        // if repeat {
        //     channel.add_flags(ChannelFlags::Sample_Loop).unwrap()
        // } else {
        //     channel.remove_flags(ChannelFlags::Sample_Loop).unwrap()
        // }
    }

    fn get_data(&self) -> Vec<tataku::FFTEntry> {
        self.data().channel
            .get_data(DataType::FFT2048, 1024)
            .unwrap_or_default()
            .into_iter()
            .map(tataku::FFTEntry::AmplitudeOnly)
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

    fn get_state(&self) -> AudioState {
        let Ok(state) = self.0.get_playback_state() 
        else { return AudioState::Unknown };
        match state {
            PlaybackState::Playing => AudioState::Playing,
            
            PlaybackState::Paused
            | PlaybackState::PausedDevice => AudioState::Paused,

            PlaybackState::Stopped 
            | PlaybackState::Stalled => AudioState::Stopped,
        }
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

    /// stream channels dont repeat
    fn set_repeat(&self, _: bool) {}


    fn get_data(&self) -> Vec<tataku::FFTEntry> {
        self.0
        .get_data(DataType::FFT2048, 1024)
        .unwrap_or_default()
        .into_iter()
        .map(tataku::FFTEntry::AmplitudeOnly)
        .collect()
    }

    fn get_duration(&self) -> f32 {
        self.0.get_length_seconds().unwrap_or_default() as f32 * 1000.0
    }
}




fn map_bass_err(e: BassError) -> errors::audio::AudioError {
    if [BassError::Empty, BassError::Fileform, BassError::Illparam].contains(&e) {
        errors::audio::AudioError::Empty
    } else {
        errors::audio::AudioError::ApiError(format!("{e:?}"))
    }
}



/// check for the bass lib
/// if not found, will be downloaded
fn check_bass() -> tataku::Result<()> {
    #[cfg(target_os = "linux")] 
    use tataku_engine_common::prelude::fs;

    #[cfg(target_os = "windows")] let filename = "bass.dll";
    #[cfg(target_os = "linux")] let filename = "libbass.so";
    #[cfg(target_os = "macos")] let filename = "libbass.dylib";

    if let Ok(mut library_path) = std::env::current_exe() {
        library_path.pop();
        library_path.push(filename);

        // check if already exists
        if library_path.exists() { return Ok(()) }
        info!("{library_path:?} not found, attempting to find or download");

        // if linux, check for lib in /usr/lib
        #[cfg(target_os = "linux")]
        if fs::exists(format!("/usr/lib/{filename}")) {
            match std::fs::copy(filename, &library_path) {
                Ok(_) => {
                    info!("Found in /usr/lib");
                    return Ok(());
                },
                Err(e) => warn!("Found in /usr/lib, but couldnt copy: {e}")
            }
        } 

        // download it from the web
        let bytes = ureq::get(format!("https://cdn.ayyeve.dev/tataku/lib/bass/{filename}"))
            .call()?
            .into_body()
            .read_to_vec()?;

        std::fs::write(&library_path, bytes)?;

        Ok(())
    } else {
        warn!("error getting current executable dir, assuming things are good...");
        Ok(())
    }
}
