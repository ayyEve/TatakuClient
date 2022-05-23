use std::fs::read_dir;
use rand::Rng;


use crate::prelude::*;
use crate::{DOWNLOADS_DIR, SONGS_DIR};

pub type DiffCalcInit = Arc<HashMap<String, f32>>;


const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;
lazy_static::lazy_static! {
    pub static ref BEATMAP_MANAGER:Arc<RwLock<BeatmapManager>> = Arc::new(RwLock::new(BeatmapManager::new()));
}

pub struct BeatmapManager {
    pub initialized: bool,

    pub current_beatmap: Option<BeatmapMeta>,
    pub beatmaps: Vec<BeatmapMeta>,
    pub beatmaps_by_hash: HashMap<String, BeatmapMeta>,

    /// previously played maps
    played: Vec<String>,
    /// current index of previously played maps
    play_index: usize,

    new_maps: Vec<BeatmapMeta>,

    /// helpful when a map is deleted
    pub(crate) force_beatmap_list_refresh: bool,

    pub on_diffcalc_complete: (MultiFuse<DiffCalcInit>, MultiBomb<DiffCalcInit>),
}
impl BeatmapManager {
    pub fn new() -> Self {
        let on_diffcalc_complete = MultiBomb::new();

        Self {
            initialized: false,

            current_beatmap: None,
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),

            played: Vec::new(),
            play_index: 0,
            new_maps: Vec::new(),

            force_beatmap_list_refresh: false,

            on_diffcalc_complete
        }
    }

    // download checking
    pub fn get_new_maps(&mut self) -> Vec<BeatmapMeta> {
        std::mem::take(&mut self.new_maps)
    }
    async fn check_downloads() {
        if read_dir(DOWNLOADS_DIR).unwrap().count() > 0 {
            extract_all().await;

            let mut folders = Vec::new();
            read_dir(SONGS_DIR)
                .unwrap()
                .for_each(|f| {
                    let f = f.unwrap().path();
                    folders.push(f.to_str().unwrap().to_owned());
                });

            for f in folders {BEATMAP_MANAGER.write().await.check_folder(&f).await}
        }

    }
    pub fn download_check_loop() {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(DOWNLOAD_CHECK_INTERVAL)).await;
                BeatmapManager::check_downloads().await;
            }
        });
    }

    /// clear the cache and db, 
    /// and do a full rescan of the songs folder
    pub async fn full_refresh(&mut self) {
        self.beatmaps.clear();
        self.beatmaps_by_hash.clear();

        Database::clear_all_maps().await;


        let mut dirs_to_check = get_settings!().external_games_folders.clone();
        dirs_to_check.push(SONGS_DIR.to_owned());


        let mut folders = Vec::new();
        for dir in dirs_to_check {
            read_dir(dir)
                .unwrap()
                .for_each(|f| {
                    let f = f.unwrap().path();
                    folders.push(f.to_str().unwrap().to_owned());
                });
        }

        for f in folders {self.check_folder(&f).await}
    }

    // adders
    pub async fn check_folder(&mut self, dir:&String) {
        if !Path::new(dir).is_dir() {return}
        let dir_files = read_dir(dir).unwrap();

        // cache of existing paths
        let mut existing_paths = HashSet::new();
        for i in self.beatmaps.iter() {
            existing_paths.insert(i.file_path.clone());
        }

        for file in dir_files {
            let file = file.unwrap().path();
            let file = file.to_str().unwrap();

            if file.ends_with(".osu") 
            || file.ends_with(".qua") 
            || file.ends_with(".adofai") 
            || file.ends_with(".ssc") 
            || file.ends_with(".sm") 
            || file.ends_with("info.txt") {
                // check file paths first
                if existing_paths.contains(file) {
                    continue
                }

                match get_file_hash(file) {
                    Ok(hash) => if self.beatmaps_by_hash.contains_key(&hash) {continue},
                    Err(e) => {
                        error!("error getting hash for file {}: {}", file, e);
                        continue;
                    }
                }

                match Beatmap::load_multiple(file.to_owned()) {
                    Ok(maps) => {
                        for map in maps {
                            let map = map.get_beatmap_meta();
                            self.add_beatmap(&map);

                            // if it got here, it shouldnt be in the database
                            // so we should add it
                            Database::insert_beatmap(&map).await;
                        }
                    }
                    Err(e) => {
                        error!("error loading beatmap '{}': {}", file, e);
                    }
                }
            }
        }
    }

    pub fn add_beatmap(&mut self, beatmap:&BeatmapMeta) {
        // check if we already have this map
        if self.beatmaps_by_hash.contains_key(&beatmap.beatmap_hash) {return debug!("map already added")}

        // dont have it, add it
        let new_hash = beatmap.beatmap_hash.clone();
        if self.initialized {self.new_maps.push(beatmap.clone())}
        self.beatmaps_by_hash.insert(new_hash, beatmap.clone());
        self.beatmaps.push(beatmap.clone());
    }


    // remover
    pub async fn delete_beatmap(&mut self, beatmap:String, game: &mut Game) {
        // delete beatmap
        self.beatmaps.retain(|b|b.beatmap_hash != beatmap);
        if let Some(old_map) = self.beatmaps_by_hash.remove(&beatmap) {
            // delete the file
            if let Err(e) = std::fs::remove_file(old_map.file_path) {
                NotificationManager::add_error_notification("Error deleting map", e).await;
            }
            // TODO: should check if this is the last beatmap in this folder
            // if so, delete the parent dir
        }

        self.force_beatmap_list_refresh = true;
        // select next one
        self.next_beatmap(game).await;
    }

    // setters
    pub async fn set_current_beatmap(&mut self, game:&mut Game, beatmap:&BeatmapMeta, _do_async:bool, use_preview_time:bool) {
        self.current_beatmap = Some(beatmap.clone());
        if let Some(map) = self.current_beatmap.clone() {
            self.played.push(map.beatmap_hash.clone());
        }

        // play song
        let audio_filename = beatmap.audio_filename.clone();
        let time = if use_preview_time {beatmap.audio_preview} else {0.0};

        // dont async with bass, causes race conditions + double audio bugs
        #[cfg(feature="neb_audio")]
        if _do_async {
            tokio::spawn(async move {
                Audio::play_song(audio_filename, false, time);
            });
        } else {
            Audio::play_song(audio_filename, false, time);
        }
        #[cfg(feature="bass_audio")]
        if let Err(e) = Audio::play_song(audio_filename, false, time).await {
            error!("Error playing song: {:?}", e);
            NotificationManager::add_text_notification("There was an error playing the audio", 5000.0, Color::RED).await;
            // Audio::stop_song();
        }

        // set bg
        game.set_background_beatmap(beatmap).await;
    }
    

    // getters
    pub fn all_by_sets(&self, _group_by: GroupBy) -> Vec<Vec<BeatmapMeta>> { // list of sets as (list of beatmaps in the set)
        
        // match group_by {
        //     GroupBy::Title => todo!(),
        //     GroupBy::Artist => todo!(),
        //     GroupBy::Creator => todo!(),
        //     GroupBy::Collections => todo!(),
        // }
        
        let mut set_map = HashMap::new();

        for beatmap in self.beatmaps.iter() {
            let key = format!("{}-{}[{}]", beatmap.artist, beatmap.title, beatmap.creator); // good enough for now
            if !set_map.contains_key(&key) {set_map.insert(key.clone(), Vec::new());}
            set_map.get_mut(&key).unwrap().push(beatmap.clone());
        }

        let mut sets = Vec::new();
        set_map.values().for_each(|e|sets.push(e.to_owned()));
        sets


    }
    pub fn get_by_hash(&self, hash:&String) -> Option<BeatmapMeta> {
        match self.beatmaps_by_hash.get(hash) {
            Some(b) => Some(b.clone()),
            None => None
        }
    }


    pub fn random_beatmap(&self) -> Option<BeatmapMeta> {
        if self.beatmaps.len() > 0 {
            let ind = rand::thread_rng().gen_range(0..self.beatmaps.len());
            let map = self.beatmaps[ind].clone();
            Some(map)
        } else {
            None
        }
    }

    pub async fn next_beatmap(&mut self, game:&mut Game) -> bool {
        self.play_index += 1;

        let next_in_queue = match self.played.get(self.play_index) {
            Some(hash) => self.get_by_hash(&hash),
            None => None
        };

        match next_in_queue {
            Some(map) => {
                self.set_current_beatmap(game, &map, false, false).await;
                // since we're playing something already in the queue, dont append it again
                self.played.pop();
                true
            }

            None => if let Some(map) = self.random_beatmap() {
                self.set_current_beatmap(game, &map, false, false).await;
                true
            } else {
                false
            }
        }

        // if self.play_index < self.played.len() {
        //     let hash = self.played[self.play_index].clone();
        //     self.get_by_hash(&hash).clone()
        // } else {
        //     self.random_beatmap()
        // }
    }
    pub async fn previous_beatmap(&mut self, game:&mut Game) -> bool {
        if self.play_index == 0 {return false}
        self.play_index -= 1;
        
        match self.played.get(self.play_index) {
            Some(hash) => {
                if let Some(map) = self.get_by_hash(&hash) {
                    self.set_current_beatmap(game, &map, false, false).await;
                    // since we're playing something already in the queue, dont append it again
                    self.played.pop();
                    true
                } else {
                    false
                }
            }
            None => false
        }
    }

    
    // changers
    pub fn update_diffs(&mut self, playmode: PlayMode, mods:&ModManager) {
        // this will be what we access and perform diff cals on
        // it will cause a momentary lagspike, 
        // but shouldnt lock everything until all diff calcs are complete
        let mut maps = self.beatmaps.clone();
        let mods = mods.clone();

        tokio::spawn(async move {
            let playmode = playmode;

            let mut existing = DifficultyDatabase::get_all_diffs(&playmode, &mods).await;
            let mut to_insert = Vec::new();

            // perform calc
            // trace!("Starting Diff Calc");
            for i in maps.iter_mut() {
                let hash = &i.beatmap_hash;
                i.diff = if let Some(diff) = existing.get(hash) { //Database::get_diff(hash, &playmode, &mods) {
                    *diff
                } else {
                    let diff = calc_diff(i, playmode.clone(), &mods).await.unwrap_or_default();
                    existing.insert(hash.clone(), diff);
                    to_insert.push(i.clone());
                    diff
                };
            }

            // insert diffs
            if to_insert.len() > 0 {
                tokio::spawn(async move {
                    DifficultyDatabase::insert_many_diffs(&playmode, &mods, to_insert.iter().map(|m| (m.beatmap_hash.clone(), m.diff))).await;
                    info!("diff calc insert done");
                });
            }
            
            {
                let mut lock = BEATMAP_MANAGER.write().await;
                lock.beatmaps = maps;
                lock.on_diffcalc_complete.0.ignite(Arc::new(existing));
            }

            // trace!("Diff calc Done");
        });
    }
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GroupBy {
    Title,
    Artist,
    Creator,
    // Difficulty,
    Collections,
}