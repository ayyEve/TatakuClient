use std::time::Instant;
use std::sync::{Arc, Weak};

use cpal::SampleFormat;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use super::fft::FFT;
use super::{AudioHandle, AudioState};
use super::sound::Sound;
use super::instance::AudioInstance;
use super::queue::{AudioQueueController, AudioQueue};

use crate::prelude::*;


pub(super) static mut SAMPLE_RATE: u32 = 0;
lazy_static::lazy_static! {
    pub static ref CURRENT_DATA: Arc<parking_lot::Mutex<Vec<f32>>> = Arc::new(parking_lot::Mutex::new(Vec::new()));
}


pub struct NebAudio {
    queue: Arc<AudioQueueController>
}
impl AudioApi for NebAudio {

    // todo: fix everything so nothing crashes and you can always change the device later etc
    fn init(_window_ptr: *mut std::ffi::c_void) -> TatakuResult<Self> where Self:Sized {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No default output device available.");
        let mut supported_configs = device.supported_output_configs().expect("Error while querying configs.");

        let supported_config_range = supported_configs.find(|thing| {
            thing.channels() == 2 && thing.sample_format() == SampleFormat::F32
        }).or_else(|| supported_configs.find(|thing| {
            thing.sample_format() == SampleFormat::F32
        })).expect("No supported config?");

        // debug!("Range Rate: {}-{}Hz", supported_config_range.min_sample_rate().0, supported_config_range.max_sample_rate().0);

        let buff_range = supported_config_range.buffer_size().clone();
        let supported_config = supported_config_range.with_max_sample_rate();
        let sample_rate = supported_config.sample_rate().0;

        let config = if let cpal::SupportedBufferSize::Range{min, max} = buff_range {
            let mut config = supported_config.config();
            config.buffer_size = cpal::BufferSize::Fixed(8192.clamp(min, max));
            trace!("setting buffer size to {}", min);
            config
        } else {
            trace!("unknown buffer size, praying to jesus");
            let config = supported_config.config();
            // config.buffer_size = cpal::BufferSize::Fixed(8192);
            config
        };

        // trace!("Sample Rate Stream: {}", sample_rate);
        let (controller, mut queue) = AudioQueue::new();

        std::thread::spawn(move || {
            let stream = device.build_output_stream(
                &config,
                move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    
                    // react to stream events and read or write stream data here.
                    let instant = Instant::now();
                    let timestamp = info.timestamp();

                    let delay = match timestamp.playback.duration_since(&timestamp.callback) {
                        Some(d) => d.as_secs_f32() * 1000.0,
                        None => {
                            // trace!("uh oh, none delay");
                            0.0
                        }
                    };

                    let mut current_data = CURRENT_DATA.lock();
                    current_data.clear();

                    queue.sync_time(instant);
                    for sample in data.iter_mut() {
                        let (raw, s) = queue.next().unwrap_or((0.0, 0.0));
                        *sample = s;

                        // if raw != 0.0 {
                        current_data.push(raw);
                        // }
                    }

                    // trace!("len: {}", current_data.len());
                    // current_data.resize(8192, 0.0);
                    // {
                    //     let mut current_data = CURRENT_DATA.lock();
                    //     current_data.fill(0.0)
                    // }

                    queue.set_delay(delay + instant.elapsed().as_secs_f32() * 1000.0);
                },
                move |err| {
                    trace!("wat: {:?}", err);
                }
            )
            .expect("Failed to build output stream.");

            stream.play().unwrap();
            std::thread::park();
        });

        unsafe { SAMPLE_RATE = sample_rate; }

        Ok(Self {
            queue: controller,
        })
    }

    fn load_stream_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn audio::AudioInstance>> {
        self.load_sample_data(data)
    }


    fn load_sample_data(&self, data: Vec<u8>) -> TatakuResult<Arc<dyn audio::AudioInstance>> {

        let sound = Sound::load_raw(data)?;

        let instance = AudioInstance::new(sound, unsafe{SAMPLE_RATE}, 1.0);
        let handle = Arc::downgrade(&instance.handle);
        self.queue.add(instance);
        Ok(Arc::new(NebAudioInstance(handle)))
    }

    fn empty_audio(&self) -> Arc<dyn audio::AudioInstance> {
        Arc::new(NebAudioInstance(Weak::new()))
    }
}

struct NebAudioInstance(Weak<AudioHandle>);

impl super::super::AudioInstance for NebAudioInstance {
    fn play(&self, _restart: bool) {
        if let Some(s) = self.0.upgrade() {
            s.play()
        }
    }

    fn pause(&self) {
        if let Some(s) = self.0.upgrade() {
            s.pause()
        }
    }

    fn stop(&self) {
        if let Some(s) = self.0.upgrade() {
            s.stop()
        }
    }

    fn is_playing(&self) -> bool {
        if let Some(s) = self.0.upgrade() {
            *s.state.lock() == AudioState::Playing
        } else {
            false
        }
    }

    fn is_paused(&self) -> bool {
        if let Some(s) = self.0.upgrade() {
            *s.state.lock() == AudioState::Paused
        } else {
            false
        }
    }

    fn is_stopped(&self) -> bool {
        if let Some(s) = self.0.upgrade() {
            *s.state.lock() == AudioState::Stopped
        } else {
            true
        }
    }

    fn get_position(&self) -> f32 {
        if let Some(s) = self.0.upgrade() {
            s.current_time()
        } else {
            0.0
        }
    }

    fn set_position(&self, pos: f32) {
        if let Some(s) = self.0.upgrade() {
            s.set_position(pos)
        }
    }

    fn set_volume(&self, vol: f32) {
        if let Some(s) = self.0.upgrade() {
            s.set_volume(vol)
        }
    }

    fn set_rate(&self, rate: f32) {
        if let Some(s) = self.0.upgrade() {
            s.set_playback_speed(rate as f64)
        }
    }

    fn get_data(&self) -> Vec<FFTData> {
        // get the audio being fed to the sound card
        let data = CURRENT_DATA.clone();
        let mut data = data.lock().clone();
        // trace!("{}", audio_data.len());

        let len = data.len();
        let size;

        if !cfg!(target_os = "linux") {
            let scale = (1024.0 / len as f32) * 8.0;
            for sample in data.iter_mut() {
                *sample *= scale;
            }
            data.resize(1024, 0.0);
            size = FFT::F1024;
        } else {
            data.resize(8192, 0.0);
            size = FFT::F8192;
        }

        let data = super::fft::fft(&mut data, size);

        data
            .into_iter()
            .filter(|(freq, _amp)| *freq < 7_000.0)
            .map(|(f, a)| FFTData::AmplitudeAndFrequency(f, a))
            .collect()

    }

    fn get_duration(&self) -> f32 {
        1.0
    }
}