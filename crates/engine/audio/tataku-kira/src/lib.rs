use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use tataku_audio::*;
use tataku_engine_common::common::*;
use tataku_engine_common::prelude as tataku;


use kira::{
    Tween,
    backend::cpal::CpalBackend,
    AudioManager as KiraAudioManager, 
    AudioManagerSettings,
	sound::{ 
        streaming::*, 
        FromFileError, 
        PlaybackState,
    },
};
const NO_TWEEN:Tween = Tween { 
    start_time: kira::StartTime::Immediate, 
    duration: Duration::ZERO, 
    easing: kira::Easing::Linear 
};

pub struct KiraAudio(Mutex<KiraAudioManager<CpalBackend>>);
impl KiraAudio {
    fn init() -> tataku::Result<Arc<dyn AudioApi>> {
        let manager = KiraAudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .map_err(tataku::Error::from_err)?;

        Ok(Arc::new(KiraAudio(Mutex::new(manager))))
    }
}
impl AudioApi for KiraAudio {
    fn load_sample_data(&self, data: Vec<u8>) -> tataku::Result<Arc<dyn AudioInstance>> {
        // TODO: StaticSoundData
        self.load_stream_data(data)
    }

    fn load_stream_data(&self, data: Vec<u8>) -> tataku::Result<Arc<dyn AudioInstance>> {
        match StreamingSoundData::from_cursor(Cursor::new(data)) {
            Ok(s) => Ok(Arc::new(
                KiraStreamAudioInstance::new(s, &mut self.0.lock())
                    .ok_or(tataku::Error::Audio(errors::audio::AudioError::Empty))?
            )),
            Err(e) => Err(tataku::Error::String(e.to_string())),
        }
    }
}

#[allow(non_upper_case_globals)]
pub const KiraAudioInit: AudioApiInit = AudioApiInit {
    name: "kira_audio",
    init: KiraAudio::init
};

struct KiraStreamAudioInstance(RwLock<StreamingSoundHandle<FromFileError>>);
impl KiraStreamAudioInstance {
    fn new(data: StreamingSoundData<FromFileError>, manager: &mut KiraAudioManager<CpalBackend>) -> Option<Self> {
        let mut handle = manager.play(data).ok()?;
        handle.pause(NO_TWEEN);
        handle.seek_to(0.0);
        Some(Self(RwLock::new(handle)))
    }
}
impl AudioInstance for KiraStreamAudioInstance {
    fn play(&self, restart: bool) {
        let mut handle = self.0.write();
        if restart {
            handle.seek_to(0.0);
        }
        handle.resume(NO_TWEEN);
    }

    fn pause(&self) {
        let mut handle = self.0.write();
        handle.pause(NO_TWEEN);
    }

    fn stop(&self) {
        println!("stopping");
        let mut handle = self.0.write();
        handle.stop(NO_TWEEN);
    }

    fn get_state(&self) -> AudioState {
        match self.0.read().state() {
            PlaybackState::Resuming
            | PlaybackState::WaitingToResume
            | PlaybackState::Playing => AudioState::Playing,
            
            PlaybackState::Pausing 
            | PlaybackState::Paused => AudioState::Paused,

            PlaybackState::Stopping
            | PlaybackState::Stopped => AudioState::Stopped,
        }
    }

    fn get_position(&self) -> f32 {
        let handle = self.0.read();
        handle.position() as f32 * 1000.0
    }

    fn set_position(&self, pos: f32) {
        let mut handle = self.0.write();
        handle.seek_to(pos as f64 / 1000.0);
    }

    fn set_volume(&self, vol: f32) {
        let mut handle = self.0.write();
        handle.set_volume(volume_to_decibels(vol), NO_TWEEN);
    }

    fn set_rate(&self, rate: f32) {
        let mut handle = self.0.write();
        handle.set_playback_rate(rate as f64, NO_TWEEN);
    }

    fn set_repeat(&self, repeat: bool) {
        if repeat {
            self.0.write().set_loop_region(..);
        } else {
            self.0.write().set_loop_region(None);
        }
    }

    fn get_data(&self) -> Vec<tataku::FFTEntry> {
        vec![]
    }

    fn get_duration(&self) -> f32 {
        1.0
    }
}

fn volume_to_decibels(percent: f32) -> f32 {
    percent.log10() * 10.0
}


// // https://github.com/WeirdConstructor/HexoDSP/blob/master/tests/common/mod.rs#L735-L783
// mod fft {
//     #[allow(unused)]
//     #[derive(Clone, Copy, Debug)]
//     pub enum FFT {
//         F16,
//         F32,
//         F64,
//         F128,
//         F512,
//         F1024,
//         F2048,
//         F4096,
//         F8192,
//         F16384,
//         F65535,
//     }

//     impl FFT {
//         pub fn size(&self) -> usize {
//             match self {
//                 FFT::F16      => 16,
//                 FFT::F32      => 32,
//                 FFT::F64      => 64,
//                 FFT::F128     => 128,
//                 FFT::F512     => 512,
//                 FFT::F1024    => 1024,
//                 FFT::F2048    => 2048,
//                 FFT::F4096    => 4096,
//                 FFT::F8192    => 8192,
//                 FFT::F16384   => 16384,
//                 FFT::F65535   => 65535,
//             }
//         }
//     }

//     /// (frequency, amplitude)
//     pub fn fft(buf: &mut [f32], size: FFT, sample_rate: f32) -> Vec<(f32, f32)> {
//         let len = size.size();
//         let mut res = vec![];

//         if len > buf.len() {
//             trace!("len > buf.len");
//             return res;
//         }

//         // Hann window:
//         for (i, s) in buf[0..len].iter_mut().enumerate() {
//             let w =
//                 0.5
//                 * (1.0 
//                 - ((2.0 * std::f32::consts::PI * i as f32)
//                     / (len as f32 - 1.0))
//                     .cos());
//             *s *= w;
//         }

//         use rustfft::{FftPlanner, num_complex::Complex};

//         let mut complex_buf =
//             buf.iter()
//             .map(|s| Complex { re: *s, im: 0.0 })
//             .collect::<Vec<Complex<f32>>>();

//         let mut p = FftPlanner::<f32>::new();
//         let fft = p.plan_fft_forward(len);


//         fft.process(&mut complex_buf[0..len]);


//         let amplitudes: Vec<_> =
//             complex_buf[0..len]
//             .iter()
//             .map(|c| c.norm())
//             .collect();
//     //    debug!("fft: {:?}", &complex_buf[0..len]);


//         for (i, amp) in amplitudes.iter().enumerate() {
//             let freq = (i as f32 * sample_rate) / len as f32;
//             if freq > 22050.0 {
//                 // no freqency images above nyquist...
//                 continue;
//             }
//     //        debug!("{:6.0} {}", freq, *amp);
//             res.push((freq.round(), *amp));
//         }

//         // debug!("fft -> len: {}, complex: {}, amplitudes: {}, res: {}", len, complex_buf.len(), amplitudes.len(), res.len());
//         res
//     }
// }



