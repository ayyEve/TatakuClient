use crate::prelude::*;

async fn load_sound(path: impl AsRef<str>, filename: String, sounds_list: &mut HashMap<String, Arc<dyn AudioInstance>>) -> bool {
    let path = path.as_ref();

    for ext in &["wav", "mp3"] {
        let path2 = format!("{path}.{ext}");
        if !Path::new(&path2).exists() { continue }

        match AudioManager::load(path2) {
            Ok(sound) => {
                // filename without extention
                // let filename = Path::new(&filename).file_stem().unwrap().to_string_lossy().to_string();
                // info!("inserting {filename}");
                sounds_list.insert(filename, sound); 
                return true
            },
            Err(TatakuError::Audio(AudioError::Empty)) => {
                // ignore these errors, just means the file provided was empty (probably)
            }
            Err(e) => NotificationManager::add_error_notification(&format!("Error loading sound {}", path), e).await,
        }
    }

    false
}

pub struct HitsoundManager {
    /// source, sound_name, sound
    sounds: HashMap<
        HitsoundSource,
        HashMap<String, Arc<dyn AudioInstance>>
    >,
    playmode_prefix: String,
}
impl HitsoundManager {
    pub fn new(playmode_prefix: String) -> Self {
        Self { sounds: HashMap::new(), playmode_prefix }
    }

    pub async fn init(&mut self, beatmap: &Arc<BeatmapMeta>) {
        let map_folder = Path::new(&beatmap.file_path).parent().unwrap();
        let map_files = map_folder.read_dir().unwrap();
        let settings = get_settings!();

        // load beatmap sounds first (if enabled)
        let mut beatmap_sounds = HashMap::new();
        if settings.beatmap_hitsounds {
            for file in map_files {
                if let Ok(file) = file {
                    let file_name = file.file_name().to_string_lossy().to_lowercase();
                    if file_name.ends_with(".wav") {
                        let filename = file_name.trim_end_matches(".wav").to_owned();
                        load_sound(file.path().to_string_lossy().trim_end_matches(".wav"), filename, &mut beatmap_sounds).await;
                    }
                }
            }
            // error!("beatmap: {:?}", beatmap_sounds.keys());
        }

        // skin and default sounds
        self.sounds.insert(HitsoundSource::Beatmap, beatmap_sounds);
        self.sounds.insert(HitsoundSource::Skin, HashMap::new());
        self.sounds.insert(HitsoundSource::Default, HashMap::new());


        let skin = settings.current_skin.clone();
        let skin_folder = format!("{SKIN_FOLDER}/{skin}");
        const SAMPLE_SETS:&[&str] = &["normal", "soft", "drum"];
        const HITSOUNDS:&[&str] = &["hitnormal", "hitwhistle", "hitfinish", "hitclap", "slidertick"];
        for sample in SAMPLE_SETS {
            for hitsound in HITSOUNDS {
                let filename = format!("{sample}-{hitsound}");
                self.load_hitsound(&skin_folder, filename).await;
            }
        }

        const OTHER_SOUNDS:&[&str] = &["combobreak"];
        for sound in OTHER_SOUNDS {
            self.load_hitsound(&skin_folder, (*sound).to_owned()).await;
        }

    } 

    async fn load_hitsound(&mut self, skin_folder: &String, filename: String) {

        // skin
        let skin_filepath = format!("{skin_folder}/{filename}");
        let skin_file = Path::new(&skin_filepath);
        if skin_file.exists() {
            let skin_sounds = self.sounds.get_mut(&HitsoundSource::Skin).unwrap();
            load_sound(skin_filepath, filename.clone(), skin_sounds).await;
        }

        // default
        let default_filepath = format!("resources/audio/{filename}");
        // let default_file = Path::new(&default_filepath);
        // if default_file.exists() {
        load_sound(default_filepath, filename.clone(), self.sounds.get_mut(&HitsoundSource::Default).unwrap()).await;
        // }
        
        // check for playmode override
        if self.playmode_prefix.len() > 0 {
            let filename = format!("{}-{filename}", self.playmode_prefix);

            // skin
            let skin_filepath = format!("{skin_folder}/{filename}");
            load_sound(skin_filepath, filename.clone(), self.sounds.get_mut(&HitsoundSource::Skin).unwrap()).await;

            // default
            let default_filepath = format!("resources/audio/{filename}");
            load_sound(default_filepath, filename.clone(), self.sounds.get_mut(&HitsoundSource::Default).unwrap()).await;
        }
    }

    pub fn play_sound(&self, hitsounds: &Vec<Hitsound>, vol: f32) {

        // The sound file is loaded from the first of the following directories that contains a matching filename:
        // Beatmap, if index is not 0
        // Skin, with the index removed
        // Default osu! resources, with the index removed
        // When filename is given, no addition sounds will be played, and this file in the beatmap directory is played instead.


        for sound in hitsounds.iter() {
            let vol = sound.volume * vol;
            let name = &sound.filename;

            // if theres no playmode prefix, dont try to play a prefixed sound first
            if self.playmode_prefix.is_empty() {
                if !self.play_sound_single(sound, None, vol) {
                    warn!("unable to play sound {name}");
                }
            } else {
                // if there is a prefix, try to play that first, otherwise try without the prefix
                if !self.play_sound_single(sound, Some(&self.playmode_prefix), vol) {
                    if !self.play_sound_single(sound, None, vol) {
                        warn!("unable to play sound {name}");
                    }
                }
            }
        }

    }

    pub fn play_sound_single(&self, sound: &Hitsound, prefix: Option<&String>, vol:f32) -> bool {
        let mut play_sound = None;
        let name = if let Some(prefix) = prefix {
            format!("{prefix}-{}", sound.filename)
        } else {
            sound.filename.clone()
        };
        // info!("attempting to play sound {name} with volume {vol}");

        for source in [
            HitsoundSource::Beatmap,
            HitsoundSource::Skin,
            HitsoundSource::Default
        ] {
            if play_sound.is_none() && sound.allowed_sources.contains(&source) {
                play_sound = self.sounds[&source].get(&name);
            }
        }

        if let Some(sound) = play_sound {
            sound.set_volume(vol);
            sound.set_position(0.0);
            sound.play(true);
            true
        } else if let Some(backup) = &sound.filename_backup {
            let name = if let Some(prefix) = prefix {
                format!("{prefix}-{backup}")
            } else {
                backup.clone()
            };
            
            for source in [
                HitsoundSource::Beatmap,
                HitsoundSource::Skin,
                HitsoundSource::Default
            ] {
                if play_sound.is_none() && sound.allowed_sources.contains(&source) {
                    play_sound = self.sounds[&source].get(&name);
                }
            }
            
            if let Some(sound) = play_sound {
                sound.set_volume(vol);
                sound.set_position(0.0);
                sound.play(true);
                true
            } else {
                false
            }

        } else {
            false
        }
    }


}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum HitsoundSource {
    Skin,
    Beatmap,
    Default
}