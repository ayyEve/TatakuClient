use std::path::Path;
use std::time::Instant;
use std::sync::{Arc, Weak};
use std::collections::HashMap;

use cpal::SampleFormat;
use parking_lot::Mutex;
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use super::AudioHandle;
use super::sound::Sound;
use crate::game::Settings;
use crate::game::managers::NotificationManager;
use super::instance::AudioInstance;
use super::queue::{AudioQueueController, AudioQueue};

use crate::prelude::*;

const SOUND_LIST:&[&'static str] = &[
    "resources/audio/don.wav",
    "resources/audio/kat.wav",
    "resources/audio/bigdon.wav",
    "resources/audio/bigkat.wav",
    "resources/audio/combobreak.mp3"
];

lazy_static::lazy_static!(
    pub static ref AUDIO: Arc<Audio> = Arc::new(Audio::setup());
    static ref CURRENT_SONG: Arc<Mutex<Option<(String,Weak<AudioHandle>)>>> = Arc::new(Mutex::new(None));

    static ref PRELOADED_SOUNDS: HashMap<String, Sound> = {
        let mut sounds:HashMap<String, Sound> = HashMap::new();

        for sound in SOUND_LIST.iter() {
            let sound_name = Path::new(sound).file_stem().unwrap().to_str().unwrap();
            trace!("loading audio file {}", sound_name);

            match Sound::load(sound) {
                Ok(sound) => {sounds.insert(sound_name.to_owned(), sound);},
                Err(e) => {
                    error!("error loading sound: {}", e);
                }
            }
        }

        sounds
    };

    pub static ref CURRENT_DATA: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    static ref PLAY_PENDING: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
);

pub struct Audio {
    queue: Arc<AudioQueueController>,
    pub sample_rate: u32,
}
impl Audio {
    // todo: fix everything so nothing crashes and you can always change the device later etc
    pub fn setup() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No default output device available.");
        let mut supported_configs = device.supported_output_configs().expect("Error while querying configs.");

        let supported_config_range = supported_configs.find(|thing|{
            thing.channels() == 2 && thing.sample_format() == SampleFormat::F32
        }).expect("No supported config?");

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

        Self {
            queue: controller,
            sample_rate
        }
    }


    pub fn play_sound(sound: Sound) -> Weak<AudioHandle> {
        let instance = AudioInstance::new(sound, AUDIO.sample_rate, 1.0);
        let handle = Arc::downgrade(&instance.handle);
        AUDIO.queue.add(instance);
        handle
    }


    pub fn play_song(path: impl AsRef<str>, restart:bool, position: f32) -> Weak<AudioHandle> {
        trace!("Playing song");
        // check if we;re already playing, if restarting is allowed
        let string_path = path.as_ref().to_owned();

        if let Some((c_path, audio)) = CURRENT_SONG.lock().clone() {
            if c_path != string_path {
                if let Some(audio) = audio.upgrade() {
                    trace!("Pre-stopping old song");
                    audio.stop();
                }
            }
        }
        
        let id = format!("{}", uuid::Uuid::new_v4());

        // set the pending song to us
        *PLAY_PENDING.lock() = id.clone();

        // load the audio data (this is what takes a million years)
        let sound = match Sound::load(path.as_ref()) {
            Ok(sound) => sound,
            Err(e) => {
                NotificationManager::add_error_notification("Error loading music file", e);
                return Weak::new()
            }
        };

        // if the pending song is no longer us, return a fake pointer
        if *PLAY_PENDING.lock() != id {
            trace!("Pending song changed, leaving");
            return Weak::new()
        }

        match CURRENT_SONG.lock().clone() {
            Some((c_path, audio)) => { // audio set
                match audio.upgrade().clone() {
                    Some(audio2) => { // exists and is playing
                        if string_path == c_path { // same file as what we want to play
                            if restart {
                                trace!("Same song, restarting"); 
                                audio2.set_position(position);
                            }
                            trace!("Same song, exiting");
                            return audio;
                        } else { // different audio
                            trace!("Stopping old song");
                            audio2.stop();
                        }
                    }
                    None => trace!("Upgrade failed"), // audio stopped
                }
            }
            None => trace!("No audio"), // no audio set
        }

        let sound = Self::play_sound(sound);
        *CURRENT_SONG.lock() = Some((string_path, sound.clone()));
        
        let upgraded = sound.upgrade().unwrap();
        upgraded.is_music.store(true, std::sync::atomic::Ordering::SeqCst); 
        upgraded.play();
        upgraded.set_volume(get_settings!().get_music_vol());
        upgraded.set_position(position);

        sound
    }
    pub fn play_song_raw(key: impl AsRef<str>, bytes: Vec<u8>) -> Weak<AudioHandle> {
        // stop current
        Audio::stop_song();

        let sound = Self::play_raw(bytes);
        let upgraded = sound.upgrade().unwrap();
        upgraded.play();
        upgraded.set_volume(get_settings!().get_music_vol());
        
        *CURRENT_SONG.lock() = Some((key.as_ref().to_owned(), sound.clone()));
        sound
    }
    
    pub fn stop_song() {
        trace!("Stopping song");
        if let Some(audio) = Audio::get_song() {
            audio.stop();
        }

        *CURRENT_SONG.lock() = None;
    }
    pub fn get_song() -> Option<Arc<AudioHandle>> {
        if let Some((_, audio)) = CURRENT_SONG.lock().clone() {
            if let Some(audio) = audio.upgrade().clone() {
                return Some(audio)
            }
        }
        None
    }
    pub fn get_song_raw() -> Option<(String, Weak<AudioHandle>)> {
        CURRENT_SONG.lock().clone()
    }

    pub fn _play(path: impl AsRef<str>) -> Weak<AudioHandle> {
        match Sound::load(path.as_ref()) {
            Ok(sound) => Audio::play_sound(sound),
            Err(e) => {
                NotificationManager::add_error_notification("Error playing music file", e);
                return Weak::new()
            }
        }
    }

    pub fn play_raw(bytes: Vec<u8>) -> Weak<AudioHandle> {
        match Sound::load_raw(bytes) {
            Ok(sound) => Audio::play_sound(sound),
            Err(e) => {
                NotificationManager::add_error_notification("Error playing music file", e);
                return Weak::new()
            }
        }
    }

    pub fn play_preloaded(name: impl AsRef<str>) -> Weak<AudioHandle> {
        PRELOADED_SOUNDS.get(name.as_ref()).map(|x| {
            let handle = Audio::play_sound(x.clone());

            let upgraded = handle.upgrade().unwrap();
            upgraded.set_volume(get_settings!().get_effect_vol());
            upgraded.play();
            handle
        }).expect("Audio was not preloaded")
    }
}
