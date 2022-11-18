use rand::Rng;
use crate::prelude::*;
use std::fs::read_dir;
use crate::{ DOWNLOADS_DIR, SONGS_DIR };

const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;
lazy_static::lazy_static! {
    pub static ref BEATMAP_MANAGER:Arc<RwLock<BeatmapManager>> = Arc::new(RwLock::new(BeatmapManager::new()));

    // lock to ensure other diffcalcs are completed first, will reduce speed over multiple calcs, but should help prevent memory overflows lol
    static ref DIFF_CALC_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

pub struct BeatmapManager {
    pub initialized: bool,

    pub current_beatmap: Option<Arc<BeatmapMeta>>,
    pub beatmaps: Vec<Arc<BeatmapMeta>>,
    pub beatmaps_by_hash: HashMap<String, Arc<BeatmapMeta>>,

    /// previously played maps
    played: Vec<Arc<BeatmapMeta>>,
    /// current index of previously played maps
    play_index: usize,

    new_maps: Vec<Arc<BeatmapMeta>>,

    /// helpful when a map is deleted
    pub(crate) force_beatmap_list_refresh: bool,

    pub new_map_added: (MultiFuse<Arc<BeatmapMeta>>, MultiBomb<Arc<BeatmapMeta>>),
}
impl BeatmapManager {
    pub fn new() -> Self {
        Self {
            initialized: false,

            current_beatmap: None,
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),

            played: Vec::new(),
            play_index: 0,
            new_maps: Vec::new(),

            force_beatmap_list_refresh: false,

            new_map_added: MultiBomb::new()
        }
    }

    fn log_played(&self) {
        for (n, i) in self.played.iter().enumerate() {
            println!("{n}. {}", i.beatmap_hash)
        }
    }

    pub async fn folders_to_check() -> Vec<std::path::PathBuf> {
        let mut folders = Vec::new();
        let mut dirs_to_check = get_settings!().external_games_folders.clone();
        dirs_to_check.push(SONGS_DIR.to_owned());

        dirs_to_check.iter()
        .map(|d| read_dir(d))
        .filter_map(|d|d.ok())
        .for_each(|f| {
            f.filter_map(|f|f.ok())
            .for_each(|p| {
                folders.push(p.path());
            })
        });

        folders
    }

    // download checking
    pub fn get_new_maps(&mut self) -> Vec<Arc<BeatmapMeta>> {
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

        let folders = Self::folders_to_check().await;
        for f in folders {
            self.check_folder(f).await
        }
    }

    // adders
    pub async fn check_folder(&mut self, dir: impl AsRef<Path>) {
        let dir = dir.as_ref();

        if !dir.is_dir() { return }
        let dir_files = read_dir(dir).unwrap();

        // cache of existing paths
        let mut existing_paths = HashSet::new();
        for i in self.beatmaps.iter() {
            existing_paths.insert(i.file_path.clone());
        }

        for file in dir_files {
            let file = file.unwrap().path();
        
            let file = match file.to_str() {
                Some(f) => f,
                None => continue,
            };

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

    pub fn add_beatmap(&mut self, beatmap:&Arc<BeatmapMeta>) {
        // check if we already have this map
        if self.beatmaps_by_hash.contains_key(&beatmap.beatmap_hash) {return debug!("map already added")}

        // dont have it, add it
        let new_hash = beatmap.beatmap_hash.clone();
        if self.initialized { 
            self.new_maps.push(beatmap.clone());
            self.new_map_added.0.ignite(beatmap.clone());
        }
        
        self.beatmaps_by_hash.insert(new_hash, beatmap.clone());
        self.beatmaps.push(beatmap.clone());
    }


    // remover
    pub async fn delete_beatmap(&mut self, beatmap:String, game: &mut Game) {
        // delete beatmap
        self.beatmaps.retain(|b|b.beatmap_hash != beatmap);
        if let Some(old_map) = self.beatmaps_by_hash.remove(&beatmap) {
            // delete the file
            if let Err(e) = std::fs::remove_file(&old_map.file_path) {
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
    pub async fn set_current_beatmap(&mut self, game:&mut Game, beatmap:&Arc<BeatmapMeta>, _do_async:bool, use_preview_time:bool) {
        trace!("Setting current beatmap to {} ({})", beatmap.beatmap_hash, beatmap.file_path);
        self.current_beatmap = Some(beatmap.clone());
        self.played.push(beatmap.clone());
        self.play_index += 1;

        // play song
        let audio_filename = beatmap.audio_filename.clone();
        let time = if use_preview_time { beatmap.audio_preview } else { 0.0 };

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
    pub fn all_by_sets(&self, _group_by: GroupBy) -> Vec<Vec<Arc<BeatmapMeta>>> { // list of sets as (list of beatmaps in the set)
        
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
    pub fn get_by_hash(&self, hash:&String) -> Option<Arc<BeatmapMeta>> {
        match self.beatmaps_by_hash.get(hash) {
            Some(b) => Some(b.clone()),
            None => None
        }
    }


    pub fn random_beatmap(&self) -> Option<Arc<BeatmapMeta>> {
        if self.beatmaps.len() > 0 {
            let ind = rand::thread_rng().gen_range(0..self.beatmaps.len());
            let map = self.beatmaps[ind].clone();
            Some(map)
        } else {
            None
        }
    }

    pub async fn next_beatmap(&mut self, game:&mut Game) -> bool {
        // println!("i: {}", self.play_index);

        match self.played.get(self.play_index + 1).cloned() {
            Some(map) => {
                self.set_current_beatmap(game, &map, false, false).await;
                // since we're playing something already in the queue, dont append it again
                self.played.pop();

                // self.log_played();

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
        if self.play_index == 0 { return false }
        // println!("i: {}", self.play_index);
        
        match self.played.get(self.play_index - 1).cloned() {
            Some(map) => {
                self.set_current_beatmap(game, &map, false, false).await;
                // since we're playing something already in the queue, dont append it again
                self.played.pop();
                // undo the index bump done in set_current_beatmap
                self.play_index -= 2; 
                
                // self.log_played();

                true
            }
            None => false
        }
    }

    
    // // changers
    // pub fn update_diffs(&mut self, playmode: PlayMode, mods:&ModManager) {
    //     if !DO_DIFF_CALC { return }

    //     warn!("Diff calc start");
    //     // this will be what we access and perform diff cals on
    //     // it will cause a momentary lagspike, 
    //     // but shouldnt lock everything until all diff calcs are complete
    //     let maps = self.beatmaps.clone();
    //     let mods = mods.clone();

    //     let calc_info = Arc::new(CalcInfo {
    //         playmode: Arc::new(playmode.clone()),
    //         mods: Arc::new(mods.clone()),
    //     });

    //     self.on_diffcalc_started.0.ignite(calc_info.clone());

    //     tokio::spawn(async move {
    //         let _ = DIFF_CALC_LOCK.lock().await;

    //         let mut existing = DifficultyDatabase::get_all_diffs(&playmode, &mods).await;
    //         let mut to_insert = HashMap::new();

    //         // perform calc
    //         // trace!("Starting Diff Calc");
    //         for i in maps {
    //             let hash = &i.beatmap_hash;
    //             if !existing.contains_key(hash) {
    //                 let diff = calc_diff(&i, playmode.clone(), &mods).await.unwrap_or_default();
    //                 existing.insert(hash.clone(), diff);
    //                 to_insert.insert(hash.clone(), diff);
    //             }
    //         }

    //         // insert diffs
    //         if to_insert.len() > 0 {
    //             DifficultyDatabase::insert_many_diffs(&playmode, &mods, to_insert.into_iter()).await;
    //         }
            
            
    //         BEATMAP_MANAGER
    //             .write()
    //             .await
    //             .on_diffcalc_completed
    //             .0
    //             .ignite(Arc::new(DiffCalcCompleteInner::new(existing, calc_info)));
            

    //         warn!("Diff calc done");
    //     });
    // }
}


#[allow(unused)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GroupBy {
    Title,
    Artist,
    Creator,
    // Difficulty,
    Collections,
}
