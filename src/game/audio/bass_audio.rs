use std::path::Path;
use crate::prelude::*;

const SOUND_LIST:&[&'static str] = &[
    "resources/audio/don.wav",
    "resources/audio/kat.wav",
    "resources/audio/bigdon.wav",
    "resources/audio/bigkat.wav",
    "resources/audio/combobreak.mp3"
];

lazy_static::lazy_static!(
    static ref CURRENT_SONG: Arc<Mutex<Option<(String, StreamChannel)>>> = Arc::new(Mutex::new(None));

    static ref PRELOADED_SOUNDS: HashMap<String, SampleChannel> = {
        let mut sounds:HashMap<String, SampleChannel> = HashMap::new();

        for sound in SOUND_LIST.iter() {
            let sound_name = Path::new(sound).file_stem().unwrap().to_str().unwrap();
            trace!("loading audio file {}", sound_name);

            match Audio::load(sound) {
                Ok(sound) => sounds.insert(sound_name.to_owned(), sound),
                Err(e) => panic!("error loading sound: {}", e),
            };
        }

        sounds
    };

    pub static ref CURRENT_DATA: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    static ref PLAY_PENDING: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
);

pub struct Audio {
    // queue: Arc<AudioQueueController>,
    // pub sample_rate: u32,
}
impl Audio {
    pub async fn play_song(path: impl AsRef<str>, restart:bool, position: f32) -> TatakuResult<StreamChannel> {
        trace!("play_song - playing {}", path.as_ref());
        // check if we're already playing, if restarting is allowed
        let string_path = path.as_ref().to_owned();

        // check if file exists
        {
            if !exists(&string_path) {
                error!("audio file does not exist! {}", string_path);
                return TatakuResult::Err(TatakuError::Audio(AudioError::FileDoesntExist))
            }
        }

        if let Some((c_path, audio)) = CURRENT_SONG.lock().await.clone() {
            if c_path != string_path {
                trace!("play_song - pre-stopping old song");
                audio.stop().unwrap();
            }
        }
        
        
        // let id = format!("{}", uuid::Uuid::new_v4());

        // // set the pending song to us
        // *PLAY_PENDING.lock() = id.clone();

        // // // load the audio data (this is what takes a million years)
        // // let sound = Audio::load_song(path.as_ref())?;

        // // if the pending song is no longer us
        // if *PLAY_PENDING.lock() != id {
        //     trace!("play_song - pending song changed, leaving");
        //     return Err(AudioError::DifferentSong.into())
        // }

        match CURRENT_SONG.lock().await.clone() {
            Some((c_path, audio)) => { // audio set
                if string_path == c_path { // same file as what we want to play
                    if restart {
                        trace!("play_song - same song, restarting"); 
                        audio.set_position(position as f64).unwrap();
                    }
                    trace!("play_song - same song, exiting");
                    return Ok(audio);
                } else { // different audio
                    trace!("play_song - stopping old song");
                    audio.stop().unwrap();
                }
            }
            None => trace!("play_song - no audio"), // no audio set
        }

        let sound = Audio::load_song(path.as_ref())?;

        // double check the song is stopped when we get here
        if let Some((_, song)) = CURRENT_SONG.lock().await.clone() {
            if let Ok(PlaybackState::Playing) = song.get_playback_state() {
                trace!("double stopping song: {}", Arc::strong_count(&song.channel.handle));
                song.stop().unwrap();
            }
        }

        sound.play(true).expect("Error playing music");
        if let Err(e) = sound.set_position(position as f64) {
            warn!("error setting position: {:?}", e);
        }
        sound.set_volume(get_settings!().get_music_vol()).unwrap();

        *CURRENT_SONG.lock().await = Some((string_path, sound.clone()));
        Ok(sound)
    }
    
    pub async fn play_song_raw(key: impl AsRef<str>, bytes: Vec<u8>) -> TatakuResult<StreamChannel> {
        // stop current
        Audio::stop_song().await;

        let sound = Audio::load_song_raw(bytes)?;
        sound.play(true).unwrap();
        sound.set_volume(get_settings!().get_music_vol()).unwrap();
        
        *CURRENT_SONG.lock().await = Some((key.as_ref().to_owned(), sound.clone()));
        Ok(sound)
    }
    
    pub async fn stop_song() {
        trace!("stopping song");
        if let Some(audio) = Audio::get_song().await {
            audio.stop().unwrap();
        }

        *CURRENT_SONG.lock().await = None;
    }
    pub async fn get_song() -> Option<StreamChannel> {
        if let Some((_, audio)) = CURRENT_SONG.lock().await.clone() {
            return Some(audio)
        }
        None
    }
    pub async fn get_song_raw() -> Option<(String, StreamChannel)> {
        CURRENT_SONG.lock().await.clone()
    }


    pub fn load_song(path: impl AsRef<str>) -> TatakuResult<StreamChannel> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::load_song_raw(bytes)
    }
    pub fn load_song_raw(bytes: Vec<u8>) -> TatakuResult<StreamChannel> {
        Ok(StreamChannel::create_from_memory(bytes, 0i32)?)
    }
    
    pub fn load(path: impl AsRef<str>) -> TatakuResult<SampleChannel> {
        let bytes = std::fs::read(path.as_ref())?;
        Ok(SampleChannel::load_from_memory(bytes, 0i32, 32)?)
    }


    pub async fn play_preloaded(name: impl AsRef<str>) -> TatakuResult<Channel> {
        match PRELOADED_SOUNDS.get(name.as_ref()).clone() {
            Some(sample) => {
                let channel = sample.clone().get_channel()?;

                channel.set_volume(get_settings!().get_effect_vol()).unwrap();
                channel.play(true).expect("Error playing sample");

                Ok(channel)
            }
            None => panic!("audio was not preloaded")
        }
    }
}