use rand::Rng;
use crate::prelude::*;
use std::fs::read_dir;

const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;
lazy_static::lazy_static! {
    pub static ref BEATMAP_MANAGER:Arc<AsyncRwLock<BeatmapManager>> = Arc::new(AsyncRwLock::new(BeatmapManager::new()));
}

pub struct BeatmapManager {
    pub initialized: bool,

    pub current_beatmap: Option<Arc<BeatmapMeta>>,
    pub beatmaps: Vec<Arc<BeatmapMeta>>,
    pub beatmaps_by_hash: HashMap<Md5Hash, Arc<BeatmapMeta>>,
    pub ignore_beatmaps: HashSet<String>,

    /// previously played maps
    played: Vec<Arc<BeatmapMeta>>,
    /// current index of previously played maps
    play_index: usize,

    /// helpful when a map is deleted
    pub force_beatmap_list_refresh: bool,
}
impl BeatmapManager {
    pub fn new() -> Self {
        GlobalValueManager::update(Arc::new(LatestBeatmap(Arc::new(BeatmapMeta::default()))));
        Self {
            initialized: false,

            current_beatmap: None,
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),
            ignore_beatmaps: HashSet::new(),

            played: Vec::new(),
            play_index: 0,

            force_beatmap_list_refresh: false,
        }
    }

    fn _log_played(&self) {
        for (n, i) in self.played.iter().enumerate() {
            println!("{n}. {}", i.beatmap_hash)
        }
    }

    pub async fn folders_to_check() -> Vec<std::path::PathBuf> {
        let mut folders = Vec::new();
        let mut dirs_to_check = Settings::get().external_games_folders.clone();
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
    async fn check_downloads() {
        if read_dir(DOWNLOADS_DIR).unwrap().count() > 0 {
            let folders = Zip::extract_all(DOWNLOADS_DIR, SONGS_DIR, ArchiveDelete::Always).await;
            info!("checking folders {folders:#?}");

            for f in folders { BEATMAP_MANAGER.write().await.check_folder(&f, true).await; }
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

        let mut new_beatmaps = Vec::new();

        info!("Reading maps");
        let folders = Self::folders_to_check().await;
        for f in folders {
            if let Some(maps) = self.check_folder(f, false).await {
                new_beatmaps.extend(maps);
            }
        }

        if !new_beatmaps.is_empty() {
            info!("Inserting maps into database");
            Database::insert_beatmaps(new_beatmaps).await;
        }
    }

    /// if this doesnt handle the database entries, returns a list of new beatmaps that should be added to the database
    pub async fn check_folder(&mut self, dir: impl AsRef<Path>, handle_database: impl Into<HandleDatabase>) -> Option<Vec<Arc<BeatmapMeta>>> {
        let dir = dir.as_ref();

        if !dir.is_dir() { return None }
        let dir_files = read_dir(dir).unwrap();

        // ignore existing paths
        let mut ignore_paths = HashSet::new();
        for i in self.beatmaps.iter() {
            ignore_paths.insert(i.file_path.clone());
        }

        // also add maps to be ignored to the list
        for i in self.ignore_beatmaps.iter() {
            ignore_paths.insert(i.clone());
        }

        let mut maps_to_add_to_database = Vec::new();

        for file in dir_files.filter_map(|s|s.ok()) {
            let file = file.path();
            let Some(file) = file.to_str() else { continue };
            // info!("checking {file}");


            if AVAILABLE_MAP_EXTENSIONS.iter().find(|e|file.ends_with(**e)).is_some() {
                // check file paths first
                if ignore_paths.contains(file) {
                    continue
                }

                match Io::get_file_hash(file) {
                    Ok(hash) => if self.beatmaps_by_hash.contains_key(&hash) { continue },
                    Err(e) => {
                        error!("error getting hash for file {file}: {e}");
                        continue;
                    }
                }

                match Beatmap::load_multiple_metadata(file) {
                    Ok(maps) => {
                        for map in maps {
                            self.add_beatmap(&map);
                            
                            // if it got here, it shouldnt be in the database
                            // so we should add it
                            maps_to_add_to_database.push(map);
                        }
                    }
                    Err(e) => {
                        error!("error loading beatmap '{file}': {e}");
                    }
                }
            }
        }

        let handle_database:HandleDatabase = handle_database.into();
        match handle_database {
            HandleDatabase::No => Some(maps_to_add_to_database),
            HandleDatabase::Yes => {
                Database::insert_beatmaps(maps_to_add_to_database).await;
                None
            }
            HandleDatabase::YesAndReturnNewMaps => {
                Database::insert_beatmaps(maps_to_add_to_database.clone()).await;
                Some(maps_to_add_to_database)
            }
        }
    }

    pub fn add_beatmap(&mut self, beatmap:&Arc<BeatmapMeta>) {
        // check if we already have this map
        if self.beatmaps_by_hash.contains_key(&beatmap.beatmap_hash) {
            // see if this beatmap is being added from another source
            if self.beatmaps.iter().find(|m|m.file_path == beatmap.file_path).is_none() { 
                trace!("adding {} to the ignore list", beatmap.file_path);
                // if so, add it to the ignore list
                self.ignore_beatmaps.insert(beatmap.file_path.clone());
                tokio::spawn(Database::add_ignored(beatmap.file_path.clone()));
            }

            return debug!("map already added") 
        }

        // dont have it, add it
        let new_hash = beatmap.beatmap_hash.clone();
        self.beatmaps_by_hash.insert(new_hash, beatmap.clone());
        self.beatmaps.push(beatmap.clone());

        if self.initialized { 
            debug!("adding beatmap {}", beatmap.version_string());
            GlobalValueManager::update(Arc::new(LatestBeatmap(beatmap.clone())));
        }
    }

    pub async fn delete_beatmap(&mut self, beatmap:Md5Hash, game: &mut Game, post_delete: PostDelete) {
        // remove beatmap from ourselves
        self.beatmaps.retain(|b|b.beatmap_hash != beatmap);

        if let Some(old_map) = self.beatmaps_by_hash.remove(&beatmap) {
            if old_map.file_path.starts_with(SONGS_DIR) {

                // delete the file
                if let Err(e) = std::fs::remove_file(&old_map.file_path) {
                    NotificationManager::add_error_notification("Error deleting map", e).await;
                }
                // TODO: should check if this is the last beatmap in this folder
                // if so, delete the parent dir
            } else {
                // file is probably in an external folder, just add this file to the ignore list
                self.ignore_beatmaps.insert(old_map.file_path.clone());
                Database::add_ignored(old_map.file_path.clone()).await;
            }
        }

        self.force_beatmap_list_refresh = true;

        if self.current_beatmap.as_ref().filter(|b|b.beatmap_hash == beatmap).is_some() {
            match post_delete {
                // select next beatmap
                PostDelete::Next => { self.next_beatmap(game).await; },
                PostDelete::Previous => { self.previous_beatmap(game).await; },
                PostDelete::Random => if let Some(map) = self.random_beatmap() {
                    self.set_current_beatmap(game, &map, true).await
                }
            }
        }
    }


    pub async fn set_current_beatmap(&mut self, game:&mut Game, beatmap:&Arc<BeatmapMeta>, use_preview_time:bool) {
        trace!("Setting current beatmap to {} ({})", beatmap.beatmap_hash, beatmap.file_path);
        GlobalValueManager::update(Arc::new(CurrentBeatmap(Some(beatmap.clone()))));
        self.current_beatmap = Some(beatmap.clone());
        self.played.push(beatmap.clone());
        self.play_index += 1;

        // update shunting yard values
        {
            let values = &mut game.shunting_yard_values;
            values.set("map.artist", beatmap.artist.clone());
            values.set("map.title", beatmap.title.clone());
            values.set("map.creator", beatmap.creator.clone());
            values.set("map.version", beatmap.version.clone());
            values.set("map.playmode", beatmap.mode.clone());
            values.set("map.game", format!("{:?}", beatmap.beatmap_type));
            values.set("map.diff_rating", 0.0f32);
            values.set("map.hash", beatmap.beatmap_hash.to_string());
        }

        // play song
        let audio_filename = beatmap.audio_filename.clone();
        let time = if use_preview_time { beatmap.audio_preview } else { 0.0 };

        if let Err(e) = AudioManager::play_song(audio_filename, false, time).await {
            error!("Error playing song: {:?}", e);
            NotificationManager::add_text_notification("There was an error playing the audio", 5000.0, Color::RED).await;
        }

        // set bg
        game.set_background_beatmap(beatmap).await;
    }
    pub async fn remove_current_beatmap(&mut self, game:&mut Game) {
        trace!("Setting current beatmap to None");
        GlobalValueManager::update(Arc::new(CurrentBeatmap(None)));
        self.current_beatmap = None;

        // stop song
        AudioManager::stop_song().await;

        // set bg
        game.remove_background_beatmap().await;
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
    pub fn get_by_hash(&self, hash:&Md5Hash) -> Option<Arc<BeatmapMeta>> {
        self.beatmaps_by_hash.get(hash).cloned()
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
                self.set_current_beatmap(game, &map, false).await;
                // since we're playing something already in the queue, dont append it again
                self.played.pop();

                true
            }

            None => if let Some(map) = self.random_beatmap() {
                self.set_current_beatmap(game, &map, false).await;
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
                self.set_current_beatmap(game, &map, false).await;
                // since we're playing something already in the queue, dont append it again
                self.played.pop();
                // undo the index bump done in set_current_beatmap
                self.play_index -= 2;

                true
            }
            None => false
        }
    }

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


crate::create_value_helper!(CurrentBeatmap, Option<Arc<BeatmapMeta>>, CurrentBeatmapHelper);
crate::create_value_helper!(LatestBeatmap, Arc<BeatmapMeta>, LatestBeatmapHelper);
crate::create_value_helper!(CurrentPlaymode, String, CurrentPlaymodeHelper);

/// this is a bad name for this
pub enum HandleDatabase {
    No,
    Yes,
    YesAndReturnNewMaps
}
impl From<bool> for HandleDatabase {
    fn from(value: bool) -> Self {
        if value {Self::Yes} else {Self::No}
    }
}