use rand::Rng;
use crate::prelude::*;
use std::fs::read_dir;

// const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;

pub struct BeatmapManager {
    pub actions: ActionQueue,

    pub initialized: bool,

    pub current_beatmap: Option<Arc<BeatmapMeta>>,
    pub beatmaps: Vec<Arc<BeatmapMeta>>,
    pub beatmaps_by_hash: HashMap<Md5Hash, Arc<BeatmapMeta>>,
    pub ignore_beatmaps: HashSet<String>,

    /// previously played maps
    played: Vec<Arc<BeatmapMeta>>,
    /// current index of previously played maps
    play_index: usize,


    // list stuff
    
    /// cache of groups before we filter them, saved from rebuilding this list every filter update
    unfiltered_groups: Vec<BeatmapGroup>,
    filtered_groups: Vec<BeatmapListGroup>,

    pub filter: String,

    selected_set: usize,
    selected_map: usize,
}
impl BeatmapManager {
    pub fn new() -> Self {
        GlobalValueManager::update(Arc::new(LatestBeatmap(Arc::new(BeatmapMeta::default()))));
        Self {
            actions: ActionQueue::new(),
            initialized: false,

            current_beatmap: None,
            beatmaps: Vec::new(),
            beatmaps_by_hash: HashMap::new(),
            ignore_beatmaps: HashSet::new(),

            played: Vec::new(),
            play_index: 0,


            unfiltered_groups: Vec::new(),
            filtered_groups: Vec::new(),
            filter: String::new(),
            selected_set: 0,
            selected_map: 0
        }
    }

    pub async fn initialize(&mut self, values: &mut ValueCollection) {
        self.initialized = true;
        self.refresh_maps(values).await;
    }

    fn _log_played(&self) {
        for (n, i) in self.played.iter().enumerate() {
            println!("{n}. {}", i.beatmap_hash)
        }
    }

    pub fn folders_to_check() -> Vec<std::path::PathBuf> {
        let mut dirs_to_check = Settings::get().external_games_folders.clone();
        dirs_to_check.push(SONGS_DIR.to_owned());

        dirs_to_check.iter()
            .map(|d| read_dir(d))
            .filter_map(|d| d.ok())
            .map(|f| f.filter_map(|f| f.ok())
            .map(|f| f.path()) )
            .flatten()
            .collect()
    }

    /// clear the cache and db, 
    /// and do a full rescan of the songs folder
    pub async fn full_refresh(&mut self) {
        self.beatmaps.clear();
        self.beatmaps_by_hash.clear();

        Database::clear_all_maps().await;

        let mut new_beatmaps = Vec::new();

        info!("Reading maps");
        let folders = Self::folders_to_check();
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

        let mut ignore_paths = self.ignore_beatmaps.clone();

        // ignore existing paths
        for i in self.beatmaps.iter() {
            ignore_paths.insert(i.file_path.clone());
        }

        let mut maps_to_add_to_database = Vec::new();

        for file in dir_files.filter_map(|s|s.ok()) {
            let file = file.path();
            let Some(file) = file.to_str() else { continue };
            // info!("checking {file}");


            if AVAILABLE_MAP_EXTENSIONS.iter().find(|e| file.ends_with(**e)).is_some() {
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
                            self.add_beatmap(&map, false).await;
                            
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

    pub async fn add_beatmap(&mut self, beatmap:&Arc<BeatmapMeta>, add_to_db: bool) {
        // check if we already have this map
        if self.beatmaps_by_hash.contains_key(&beatmap.beatmap_hash) {
            // see if this beatmap is being added from another source
            if self.beatmaps.iter().find(|m| m.file_path == beatmap.file_path).is_none() { 
                // if so, add it to the ignore list
                trace!("adding {} to the ignore list, as it already exists", beatmap.file_path);
                self.ignore_beatmaps.insert(beatmap.file_path.clone());
                tokio::spawn(Database::add_ignored(beatmap.file_path.clone()));
            }

            debug!("map already added");
            return;
        }

        // dont have it, add it
        let new_hash = beatmap.beatmap_hash.clone();
        self.beatmaps_by_hash.insert(new_hash, beatmap.clone());
        self.beatmaps.push(beatmap.clone());

        if self.initialized { 
            debug!("adding beatmap {}", beatmap.version_string());
            //TODO:!!!! move this to values
            // global.new_map_hash
            GlobalValueManager::update(Arc::new(LatestBeatmap(beatmap.clone())));
        }

        if add_to_db {
            Database::insert_beatmaps(vec![beatmap.clone()]).await;
        }

    }

    pub async fn delete_beatmap(&mut self, beatmap:Md5Hash, values: &mut ValueCollection, post_delete: PostDelete) {
        // remove beatmap from ourselves
        self.beatmaps.retain(|b| b.beatmap_hash != beatmap);

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

        if self.current_beatmap.as_ref().filter(|b|b.beatmap_hash == beatmap).is_some() {
            match post_delete {
                // select next beatmap
                PostDelete::Next => { self.next_beatmap(values).await; },
                PostDelete::Previous => { self.previous_beatmap(values).await; },
                PostDelete::Random => if let Some(map) = self.random_beatmap() {
                    self.set_current_beatmap(values, &map, true, true).await
                }
            }
        }
    }

    #[async_recursion::async_recursion]
    pub async fn set_current_beatmap(
        &mut self, 
        // game:&mut Game, 
        values: &mut ValueCollection,
        beatmap:&Arc<BeatmapMeta>, 
        use_preview_time:bool, 
        restart_song: bool
    ) {
        trace!("Setting current beatmap to {} ({})", beatmap.beatmap_hash, beatmap.file_path);
        GlobalValueManager::update(Arc::new(CurrentBeatmap(Some(beatmap.clone()))));
        self.current_beatmap = Some(beatmap.clone());
        self.played.push(beatmap.clone());
        self.play_index += 1;

        // update shunting yard values
        {
            let map: CustomElementValue = beatmap.deref().into();
            let mut map = map.as_map_helper().unwrap();

            let mode = values.get_string("global.playmode").unwrap_or("osu".to_owned());
            let actual_mode = beatmap.check_mode_override(mode.clone());


            let mods = values.try_get::<ModManager>("global.mods").unwrap_or_default();
            let diff = get_diff(&beatmap, &actual_mode, &mods);
            map.set("diff_rating", diff.unwrap_or(0.0));

            if let Some(info) = get_gamemode_info(&mode) { 
                let diff_meta = BeatmapMetaWithDiff::new(beatmap.clone(), diff);
                let diff_info = info.get_diff_string(&diff_meta, &mods);
                map.set("diff_info", diff_info);
            } else {
                map.set("diff_info", String::new());
            }

            values.set("map", map.finish());

            
            values.set("global.playmode_actual", &actual_mode);
            values.set("global.playmode_actual_display", gamemode_display_name(&actual_mode));
        }

        // play song
        let audio_filename = beatmap.audio_filename.clone();
        let time = if use_preview_time { beatmap.audio_preview } else { 0.0 };

        // set the song
        self.actions.push(SongAction::Set(SongMenuSetAction::FromFile(audio_filename, SongPlayData {
            play: true,
            restart: restart_song,
            position: Some(time),
            volume: Some(Settings::get().get_music_vol()),
            ..Default::default()
        })));
        // make sure the song is playing
        self.actions.push(SongAction::Play);

        // if let Err(e) = AudioManager::play_song(audio_filename, false, time).await {
        //     error!("Error playing song: {:?}", e);
        //     NotificationManager::add_text_notification("There was an error playing the audio", 5000.0, Color::RED).await;
        // }

        // // set bg
        // game.set_background_beatmap(beatmap).await;
    }
    #[async_recursion::async_recursion]
    pub async fn remove_current_beatmap(&mut self, values: &mut ValueCollection) {
        trace!("Setting current beatmap to None");
        // GlobalValueManager::update(Arc::new(CurrentBeatmap(None)));
        self.current_beatmap = None;

        // stop song
        self.actions.push(SongAction::Stop);
        // AudioManager::stop_song().await;

        // remove the map from the game's values as well
        values.remove("map");

        // // set bg
        // game.remove_background_beatmap().await;
    }
    

    // getters
    pub fn all_by_sets(&self, _group_by: GroupBy) -> Vec<BeatmapGroup> {
        let mut set_map: HashMap<BeatmapGroupValue, BeatmapGroup> = HashMap::new();

        for beatmap in self.beatmaps.iter().cloned() {
            let key = format!("[{}] // {}-{}", beatmap.creator, beatmap.artist, beatmap.title);
            let key = BeatmapGroupValue::Set(key);

            if let Some(list) = set_map.get_mut(&key) {
                list.maps.push(beatmap);
            } else {
                let mut group = BeatmapGroup::new(key.clone());
                group.maps.push(beatmap);
                set_map.insert(key, group);
            }

            // set_map
            //     .entry(key.clone())
            //     .or_insert_with(|| BeatmapGroup::default())

            // if !set_map.contains_key(&key) { set_map.insert(key.clone(), Vec::new()) }
            // set_map.get_mut(&key).unwrap().push(beatmap.clone());
        }

        set_map.into_values().collect()
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

    pub async fn next_beatmap(&mut self, values:&mut ValueCollection) -> bool {
        // println!("i: {}", self.play_index);

        match self.played.get(self.play_index + 1).cloned() {
            Some(map) => {
                self.set_current_beatmap(values, &map, false, true).await;
                // since we're playing something already in the queue, dont append it again
                self.played.pop();

                true
            }

            None => if let Some(map) = self.random_beatmap() {
                self.set_current_beatmap(values, &map, false, true).await;
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

    pub async fn previous_beatmap(&mut self, values:&mut ValueCollection) -> bool {
        if self.play_index == 0 { return false }
        // println!("i: {}", self.play_index);
        
        match self.played.get(self.play_index - 1).cloned() {
            Some(map) => {
                self.set_current_beatmap(values, &map, false, true).await;
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


impl BeatmapManager {
    
    pub async fn refresh_maps(&mut self, values: &mut ValueCollection) {
        trace!("Refreshing maps");

        let group_by = values.try_get::<GroupBy>("settings.group_by").unwrap_or_default();
        //TODO: allow grouping by not just map set
        self.unfiltered_groups = self.all_by_sets(group_by);

        self.apply_filter(values).await;
    }
    
    pub async fn apply_filter(&mut self, values: &mut ValueCollection) {
        trace!("Applying Filter");
        self.filtered_groups.clear();
        
        // get filter text and split here so we arent splitting every map
        let filter_text = self.filter.to_ascii_lowercase();
        let filters = filter_text.split(" ").filter(|s| !s.is_empty()).collect::<Vec<_>>();

        let Ok(Ok(mods)) = values.get_raw("global.mods").map(ModManager::try_from) else { return };
        let Ok(mode) = values.get_string("global.playmode") else { return }; 

        for group in self.unfiltered_groups.iter() {
            let mut maps = group.maps.iter().map(|m| {
                let mode = m.check_mode_override(mode.clone());
                let diff = get_diff(&m, &mode, &mods);

                BeatmapMetaWithDiff::new(m.clone(), diff)
            }).collect::<Vec<_>>();

            // apply filter
            if !filters.is_empty() {
                for filter in filters.iter() {
                    maps.retain(|bm| bm.filter(filter));
                }

                if maps.is_empty() { continue }
            }

            let name = group.get_name().clone();
            self.filtered_groups.push(BeatmapListGroup { maps, number: 0, name });
        }

        self.sort(values)
    }

    pub fn sort(&mut self, values: &mut ValueCollection) {
        let current_hash = values.try_get("map.hash").unwrap_or_default();

        // sort
        macro_rules! sort {
            ($property:tt, String) => {
                self.filtered_groups.sort_by(|a, b| a.maps[0].$property.to_lowercase().cmp(&b.maps[0].$property.to_lowercase()))
            };
            ($property:ident, Float) => {
                self.filtered_groups.sort_by(|a, b| a.maps[0].$property.partial_cmp(&b.maps[0].$property).unwrap())
            }
        }

        let Ok(sort_by) = values.try_get::<SortBy>("settings.sort_by") else { return };

        match sort_by {
            SortBy::Title => sort!(title, String),
            SortBy::Artist => sort!(artist, String),
            SortBy::Creator => sort!(creator, String),
            SortBy::Difficulty => sort!(diff, Float),
        }
            
        let mut selected = false;
        for (n, i) in self.filtered_groups.iter_mut().enumerate() {
            i.number = n;

            // make sure we have the correct selected set and map number
            if !selected {
                if let Some(j) = i.has_hash(&current_hash) {
                    self.selected_set = n;
                    self.selected_map = j;
                    selected = true;
                }
            }
        }

        self.update_values(values, current_hash);
    }



    pub fn update_values(&mut self, values: &mut ValueCollection, current_hash: Md5Hash) {
        let filtered_groups = CustomElementValue::List(
            self.filtered_groups
                .iter()
                .map(|group| group.into_map(current_hash)).collect()
        );

        values.set("beatmap_list.groups", filtered_groups);
    }

    pub fn select_set(&mut self, set_num: usize, values: &mut ValueCollection) {
        debug!("selecting set: {set_num}");
        
        self.selected_set = set_num;
        self.select_map(0, values);

        self.actions.push(TatakuAction::PerformOperation(
            snap_to_id(
            "beatmap_scroll", 
            iced::widget::scrollable::RelativeOffset { 
                x: 0.0,
                y: set_num as f32 / self.filtered_groups.len() as f32
            })
        ))
    }
    pub fn next_set(&mut self, values: &mut ValueCollection) {
        self.select_set(self.selected_set.wrapping_add_1(self.filtered_groups.len()), values)
    }
    pub fn prev_set(&mut self, values: &mut ValueCollection) {
        self.select_set(self.selected_set.wrapping_sub_1(self.filtered_groups.len()), values)
    }

    pub fn select_map(&mut self, map_num: usize, values: &mut ValueCollection)  {
        self.selected_map = map_num;

        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        if let Some(map) = set.maps.get(self.selected_map) {
            self.actions.push(BeatmapAction::Set(map.meta.clone(), SetBeatmapOptions::new().use_preview_point(true)));
            self.update_values(values, map.beatmap_hash);
        }

    }
    pub fn next_map(&mut self, values: &mut ValueCollection) {
        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_add_1(set.maps.len()), values)
    }
    pub fn prev_map(&mut self, values: &mut ValueCollection) {
        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_sub_1(set.maps.len()), values)
    }
}


#[allow(unused)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum GroupBy {
    #[default]
    Set,
    Collections,
}
impl GroupBy {
    pub fn list() -> Vec<Self> {
        vec![
            Self::Set,
            Self::Collections,
        ]
    }
}
impl TryFrom<&CustomElementValue> for GroupBy {
    type Error = String;
    fn try_from(value: &CustomElementValue) -> Result<Self, Self::Error> {
        match value {
            CustomElementValue::String(s) => {
                match &**s {
                    "Set" | "set" => Ok(Self::Set),
                    "Collections" | "collections" => Ok(Self::Collections),
                    other => Err(format!("invalid GroupBy str: '{other}'"))
                }
            }
            CustomElementValue::U64(n) => {
                match *n {
                    0 => Ok(Self::Set),
                    1 => Ok(Self::Collections),
                    other => Err(format!("Invalid GroupBy number: {other}")),
                }
            }

            other => Err(format!("Invalid GroupBy value: {other:?}"))
        }
    }
}
impl Into<CustomElementValue> for GroupBy {
    fn into(self) -> CustomElementValue {
        CustomElementValue::String(format!("{self:?}"))
    }
}




crate::create_value_helper!(CurrentBeatmap, Option<Arc<BeatmapMeta>>, CurrentBeatmapHelper);
crate::create_value_helper!(LatestBeatmap, Arc<BeatmapMeta>, LatestBeatmapHelper);
// crate::create_value_helper!(CurrentPlaymode, String, CurrentPlaymodeHelper);

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


/// A group of beatmaps
pub struct BeatmapGroup {
    pub group_value: BeatmapGroupValue,
    pub maps: Vec<Arc<BeatmapMeta>>,
}
impl BeatmapGroup {
    pub fn new(group: BeatmapGroupValue) -> Self {
        Self {
            group_value: group,
            maps: Vec::new()
        }
    }

    pub fn get_name(&self) -> &String {
        self.group_value.get_name()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BeatmapGroupValue {
    Set(String),
    Collection(String),
}
impl BeatmapGroupValue {
    pub fn get_name(&self) -> &String {
        match self {
            Self::Set(name) => name,
            Self::Collection(name) => name,
        }
    }
}




pub struct BeatmapListGroup {
    pub number: usize,
    pub name: String,
    pub maps: Vec<BeatmapMetaWithDiff>,
}
impl BeatmapListGroup {
    fn has_hash(&self, hash: &Md5Hash) -> Option<usize> {
        if let Some((i,_)) = self.maps.iter().enumerate().find(|(_,b)| b.comp_hash(*hash)) {
            return Some(i)
        } 
        None
    }
    pub fn into_map(&self, current_hash: Md5Hash) -> CustomElementValue {
        let mut is_selected = false;
        
        let maps:Vec<CustomElementValue> = self.maps.iter().map(|beatmap| {
            let map_is_selected = beatmap.comp_hash(current_hash);
            if map_is_selected { is_selected = true }

            let map:CustomElementValue = beatmap.deref().deref().into();
            let mut map = map.as_map_helper().unwrap();
            
            map.set("diff_rating", beatmap.diff.unwrap_or_default());
            map.set("is_selected", map_is_selected);
            map.finish()
        }).collect();

        let mut group = CustomElementMapHelper::default();
        group.set("maps", maps);
        group.set("selected", is_selected);
        group.set("name", self.name.clone());
        group.set("id", self.number as u64);
        
        group.finish()
    }
}
