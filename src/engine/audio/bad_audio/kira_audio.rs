use std::io::Cursor;
use crate::prelude::*;

use kira::{
    tween::Tween,
	manager::{
		AudioManager as KiraAudioManager, 
        AudioManagerSettings,
		backend::cpal::CpalBackend
	},
	sound::{ 
        streaming::*, 
        FromFileError, 
        static_sound::PlaybackState
    },
};
const NO_TWEEN:Tween = Tween { start_time: kira::StartTime::Immediate, duration: Duration::ZERO, easing: kira::tween::Easing::Linear };

pub struct KiraAudio(Arc<parking_lot::Mutex<KiraAudioManager<CpalBackend>>>);
impl AudioApi for KiraAudio {
    fn init() -> TatakuResult<Self> where Self:Sized {
        let manager = KiraAudioManager::<CpalBackend>::new(AudioManagerSettings::default())
        .map_err(|e|TatakuError::String(e.to_string()))?;




        Ok(Self(Arc::new(parking_lot::Mutex::new(manager))))
    }

    fn load_sample_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        self.load_stream_data(data)
    }

    fn load_stream_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn AudioInstance>> {
        match StreamingSoundData::from_cursor(Cursor::new(data), StreamingSoundSettings::default()) {
            Ok(s) => Ok(Arc::new(KiraStreamAudioInstance::new(s, self.0.clone()))),
            Err(e) => Err(TatakuError::String(e.to_string()))
        }
    }

    fn empty_audio(&self) -> Arc<dyn AudioInstance> {
        self.load_stream_data(vec![0x52,0x49,0x46,0x46,0x28,0x00,0x00,0x00,0x57,0x41,0x56,0x45,0x66,0x6D,0x74,0x20,0x10,0x00,0x00,0x00,0x01,0x00,0x02,0x00,0x44,0xAC,0x00,0x00,0x88,0x58,0x01,0x00,0x02,0x00,0x08,0x00,0x64,0x61,0x74,0x61,0x04,0x00,0x00,0x00,0x80,0x80,0x80,0x80]).unwrap()
    }
}

struct StreamData {
    handle: Option<> 
}

struct KiraStreamAudioInstance (Arc<parking_lot::Mutex<StreamingSoundHandle<FromFileError>>>, Arc<parking_lot::Mutex<KiraAudioManager<CpalBackend>>>);
impl KiraStreamAudioInstance {
    fn new(data: StreamingSoundData<FromFileError>, manager: Arc<parking_lot::Mutex<KiraAudioManager<CpalBackend>>>) -> Self {
        let mut manager_locked = manager.lock();
        let mut handle = manager_locked.play(data).expect("pain and suffering");
        let _ = handle.pause(NO_TWEEN);
        let _ = handle.seek_to(0.0);
        drop(manager_locked);

        Self(Arc::new(parking_lot::Mutex::new(handle)), manager)
    }
}
impl AudioInstance for KiraStreamAudioInstance {
    fn play(&self, restart: bool) {
        let mut handle = self.0.lock();
        if restart {
            let _ = handle.seek_to(0.0);
        }
        let _ = handle.resume(NO_TWEEN);
    }

    fn pause(&self) {
        let mut handle = self.0.lock();
        let _ = handle.pause(NO_TWEEN);
    }

    fn stop(&self) {
        println!("stopping");
        let mut handle = self.0.lock();
        let _ = handle.stop(NO_TWEEN);
    }

    fn is_playing(&self) -> bool {
        let handle = self.0.lock();
        handle.state() == PlaybackState::Playing
    }

    fn is_paused(&self) -> bool {
        let handle = self.0.lock();
        handle.state() == PlaybackState::Paused || handle.state() == PlaybackState::Pausing
    }

    fn is_stopped(&self) -> bool {
        let handle = self.0.lock();
        handle.state() == PlaybackState::Stopped || handle.state() == PlaybackState::Stopping
    }

    fn get_position(&self) -> f32 {
        let handle = self.0.lock();
        handle.position() as f32 * 1000.0
    }

    fn set_position(&self, pos: f32) {
        let mut handle = self.0.lock();
        let _ = handle.seek_to(pos as f64 / 1000.0);
    }

    fn set_volume(&self, vol: f32) {
        let mut handle = self.0.lock();
        let _ = handle.set_volume(vol as f64, NO_TWEEN);
    }

    fn set_rate(&self, rate: f32) {
        let mut handle = self.0.lock();
        let _ = handle.set_playback_rate(rate as f64, NO_TWEEN);
    }

    fn get_data(&self) -> Vec<FFTData> {
        vec![]
    }

    fn get_duration(&self) -> f32 {
        1.0
    }
}

impl Drop for KiraStreamAudioInstance {
    fn drop(&mut self) {
        if Arc::strong_count(&self.0) == 0 {
            let _ = self.0.lock().stop(NO_TWEEN);
        }
    }
}

    // https://github.com/WeirdConstructor/HexoDSP/blob/master/tests/common/mod.rs#L735-L783
mod fft {
    #[allow(unused)]
    #[derive(Clone, Copy, Debug)]
    pub enum FFT {
        F16,
        F32,
        F64,
        F128,
        F512,
        F1024,
        F2048,
        F4096,
        F8192,
        F16384,
        F65535,
    }

    impl FFT {
        pub fn size(&self) -> usize {
            match self {
                FFT::F16      => 16,
                FFT::F32      => 32,
                FFT::F64      => 64,
                FFT::F128     => 128,
                FFT::F512     => 512,
                FFT::F1024    => 1024,
                FFT::F2048    => 2048,
                FFT::F4096    => 4096,
                FFT::F8192    => 8192,
                FFT::F16384   => 16384,
                FFT::F65535   => 65535,
            }
        }
    }

    /// (frequency, amplitude)
    pub fn fft(buf: &mut [f32], size: FFT, sample_rate: f32) -> Vec<(f32, f32)> {
        let len = size.size();
        let mut res = vec![];

        if len > buf.len() {
            trace!("len > buf.len");
            return res;
        }

        // Hann window:
        for (i, s) in buf[0..len].iter_mut().enumerate() {
            let w =
                0.5
                * (1.0 
                - ((2.0 * std::f32::consts::PI * i as f32)
                    / (len as f32 - 1.0))
                    .cos());
            *s *= w;
        }

        use rustfft::{FftPlanner, num_complex::Complex};

        let mut complex_buf =
            buf.iter()
            .map(|s| Complex { re: *s, im: 0.0 })
            .collect::<Vec<Complex<f32>>>();

        let mut p = FftPlanner::<f32>::new();
        let fft = p.plan_fft_forward(len);


        fft.process(&mut complex_buf[0..len]);


        let amplitudes: Vec<_> =
            complex_buf[0..len]
            .iter()
            .map(|c| c.norm())
            .collect();
    //    debug!("fft: {:?}", &complex_buf[0..len]);


        for (i, amp) in amplitudes.iter().enumerate() {
            let freq = (i as f32 * sample_rate) / len as f32;
            if freq > 22050.0 {
                // no freqency images above nyquist...
                continue;
            }
    //        debug!("{:6.0} {}", freq, *amp);
            res.push((freq.round(), *amp));
        }

        // debug!("fft -> len: {}, complex: {}, amplitudes: {}, res: {}", len, complex_buf.len(), amplitudes.len(), res.len());
        res
    }
}



