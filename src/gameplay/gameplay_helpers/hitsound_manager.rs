use crate::prelude::*;

async fn load_sound(path: impl AsRef<str>, filename: String, sounds_list: &mut HashMap<String, SampleChannel>) {
    match Audio::load(path.as_ref()) {
        Ok(sound) => { sounds_list.insert(filename, sound); },
        Err(e) => NotificationManager::add_error_notification(&format!("Error loading sound {}", path.as_ref()), e).await,
    }
}

pub struct HitsoundManager {
    /// source, sound_name, sound
    sounds: HashMap<
        HitsoundSource,
        HashMap<String, SampleChannel>
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


        // load beatmap sounds first
        let mut beatmap_sounds = HashMap::new();
        for file in map_files {
            if let Ok(file) = file {
                let file_name = file.file_name().to_string_lossy().to_lowercase();
                if file_name.ends_with(".wav") {
                    load_sound(file.path().to_string_lossy(), file_name, &mut beatmap_sounds).await;
                }
            }
        }

        // skin and default sounds
        let mut skin_sounds = HashMap::new();
        let mut default_sounds = HashMap::new();


        let skin = get_settings!().current_skin.clone();
        let skin_folder = format!("{SKIN_FOLDER}/{skin}");
        const SAMPLE_SETS:&[&str] = &["normal", "soft", "drum"];
        const HITSOUNDS:&[&str] = &["normal", "whistle", "finish", "clap"];

        for sample in SAMPLE_SETS {
            for hitsound in HITSOUNDS {
                let filename = format!("{sample}-hit{hitsound}.wav");

                // skin
                let skin_filepath = format!("{skin_folder}/{filename}");
                let skin_file = Path::new(&skin_filepath);
                if skin_file.exists() {
                    load_sound(skin_filepath, filename.clone(), &mut skin_sounds).await;
                }

                // default
                let default_filepath = format!("resources/audio/{filename}");
                let default_file = Path::new(&default_filepath);
                if default_file.exists() {
                    load_sound(default_filepath, filename.clone(), &mut default_sounds).await;
                }
                
                if self.playmode_prefix.len() > 0 {
                    let filename = format!("{}-{sample}-hit{hitsound}.wav", self.playmode_prefix);

                    // skin
                    let skin_filepath = format!("{skin_folder}/{filename}");
                    let skin_file = Path::new(&skin_filepath);
                    if skin_file.exists() {
                        load_sound(skin_filepath, filename.clone(), &mut skin_sounds).await;
                    }

                    // default
                    let default_filepath = format!("resources/audio/{filename}");
                    let default_file = Path::new(&default_filepath);
                    if default_file.exists() {
                        load_sound(default_filepath, filename.clone(), &mut default_sounds).await;
                    }
                }
            }
        }
        // error!("beatmap: {:?}", beatmap_sounds.keys());
        // error!("skin: {:?}", skin_sounds.keys());
        // error!("default: {:?}", default_sounds.keys());

        self.sounds.insert(HitsoundSource::Skin, skin_sounds);
        self.sounds.insert(HitsoundSource::Beatmap, beatmap_sounds);
        self.sounds.insert(HitsoundSource::Default, default_sounds);
    } 

    // TODO: completely redo this
    pub fn play_sound(&self, note_hitsound: u8, note_hitsamples:HitSamples, vol: f32, normal_by_default: bool) {
        let mut play_normal = normal_by_default || (note_hitsound & 1) > 0; // 0: Normal
        let mut play_whistle = (note_hitsound & 2) > 0; // 1: Whistle
        let mut play_finish = (note_hitsound & 4) > 0; // 2: Finish
        let mut play_clap = (note_hitsound & 8) > 0; // 3: Clap

        // get volume
        // let mut vol = (if note_hitsamples.volume == 0 {timing_point.volume} else {note_hitsamples.volume} as f32 / 100.0) * self.settings.get_effect_vol();
        // if self.menu_background {vol *= self.background_game_settings.hitsound_volume};


        // https://osu.ppy.sh/wiki/en/osu%21_File_Formats/Osu_%28file_format%29#hitsounds

        // normalSet and additionSet can be any of the following:
        // 0: No custom sample set
        // For normal sounds, the set is determined by the timing point's sample set.
        // For additions, the set is determined by the normal sound's sample set.
        // 1: Normal set
        // 2: Soft set
        // 3: Drum set

        // The filename is <sampleSet>-hit<hitSound><index>.wav, where:

        // sampleSet is normal, soft, or drum, determined by either normalSet or additionSet depending on which hitsound is playing
        const SAMPLE_SETS:&[&str] = &["normal", "normal", "soft", "drum"];
        // hitSound is normal, whistle, finish, or clap
        // index is the same index as above, except it is not written if the value is 0 or 1

        // (filename, index)
        let mut play_list = Vec::new();

        // if the hitsound is being overridden
        if let Some(name) = note_hitsamples.filename {
            if name.len() > 0 {
                #[cfg(feature="debug_hitsounds")]
                debug!("got custom sound: {}", name);
                if exists(format!("resources/audio/{}", name)) {
                    play_normal = (note_hitsound & 1) > 0;
                    play_whistle = false;
                    play_clap = false;
                    play_finish = false;
                    warn!("playing custom sound {name}");

                    play_list.push(name)
                } else {
                    #[cfg(feature="debug_hitsounds")]
                    warn!("doesnt exist");
                }
            }
        }


        if play_normal {
            let sample_set = SAMPLE_SETS[note_hitsamples.normal_set as usize];
            let hitsound = format!("{sample_set}-hitnormal");
            play_list.push(hitsound)
        }

        if play_whistle {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitwhistle", sample_set);
            play_list.push(hitsound)
        }
        if play_finish {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitfinish", sample_set);
            play_list.push(hitsound)
        }
        if play_clap {
            let sample_set = SAMPLE_SETS[note_hitsamples.addition_set as usize];
            let hitsound = format!("{}-hitclap", sample_set);
            play_list.push(hitsound)
        }


        // The sound file is loaded from the first of the following directories that contains a matching filename:
        // Beatmap, if index is not 0
        // Skin, with the index removed
        // Default osu! resources, with the index removed
        // When filename is given, no addition sounds will be played, and this file in the beatmap directory is played instead.


        for sound in play_list {
            // if theres no playmode prefix, dont try to play a prefixed sound first
            if self.playmode_prefix.is_empty() {
                if !self.play_sound_single(&sound, note_hitsamples.index, vol) {
                    warn!("unable to play sound {sound}");
                }
            } else {
                // if there is a prefix, try to play that first, otherwise try without the prefix
                if !self.play_sound_single(&format!("{}-{sound}", self.playmode_prefix), note_hitsamples.index, vol) {
                    if !self.play_sound_single(&sound, note_hitsamples.index, vol) {
                        warn!("unable to play sound {sound}");
                    }
                }
            }
        }

    }

    fn play_sound_single(&self, sound: &String, index: u8, vol:f32) -> bool {
        let mut play_sound = None;
        
        // check beatmap if index is not 0
        if index != 0 {
            let sound = if index > 1 {
                format!("{sound}{index}")
            } else {
                sound.clone()
            };

            let sound = format!("{sound}.wav");
            play_sound = self.sounds[&HitsoundSource::Beatmap].get(&sound);
            // if play_sound.is_some() {warn!("playing {sound} from beatmap")}
        }
        let sound = format!("{sound}.wav");

        // try skin
        if play_sound.is_none() {
            play_sound = self.sounds[&HitsoundSource::Skin].get(&sound);
            // if play_sound.is_some() {warn!("playing {sound} from skin")}
        }

        // try default
        if play_sound.is_none() {
            play_sound = self.sounds[&HitsoundSource::Default].get(&sound);
            // if play_sound.is_some() {warn!("playing {sound} from resources")}
        }

        if let Some(sound) = play_sound {
            #[cfg(feature="bass_audio")] {
                sound.set_volume(vol).unwrap();
                sound.set_position(0.0).unwrap();
                sound.play(true).unwrap();
            }
            #[cfg(feature="neb_audio")] {
                let sound = Audio::play_sound(sound.clone());
                if let Some(sound) = sound.upgrade() {
                    sound.set_volume(vol);
                    sound.set_position(0.0);
                    sound.play();
                }
            }
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum HitsoundSource {
    Skin,
    Beatmap,
    Default
}