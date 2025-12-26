use rand::Rng;
use crate::prelude::*;
use std::fs::read_dir;
use tataku::WrappingClamp;

use common::{
    Md5Hash,
    reflect::*,
};

use engine::{
    actions,
    Notification,
    database::DifficultyProvider,
    data::{
        SortBy,
        GroupBy,
        BeatmapCollection,
    },
    beatmaps::{
        Beatmap,
        BeatmapMeta,
    },
    gameplay::{
        GamemodeInfos,
        mods::Mods,
        difficulty_value::GetDiffValue,
    },
};


#[derive(Reflect)]
#[reflect(dont_clone)]
#[derive(Default, Debug)]
pub struct BeatmapManager {
    #[reflect(skip)] pub actions: actions::ActionQueue,
    #[reflect(skip)] pub initialized: bool,
    #[reflect(skip)] infos: GamemodeInfos,

    #[reflect(alias("current"))]
    pub current_beatmap: Option<Md5Hash>,

    #[reflect(skip)] pub ignore_beatmaps: HashSet<ArcStr>,
    diffs: HashMap<Md5Hash, BeatmapDifficulty>,
    pub beatmaps: HashMap<Md5Hash, Arc<BeatmapMeta>>,

    pub collections: Vec<BeatmapCollection>,

    /// previously played maps
    played: Vec<Md5Hash>,
    /// current index of previously played maps
    play_index: usize,

    // list stuff
    pub filter_text: String,

    /// cache of groups before we filter them, saves from rebuilding this list every filter update
    #[reflect(skip)] unfiltered_groups: Vec<BeatmapGroup>,
    groups: Vec<BeatmapListGroup>,

    selected_set: usize,
    selected_map: usize,
}
impl BeatmapManager {
    pub fn new(infos: GamemodeInfos) -> Self {
        Self {
            infos,
            actions: actions::ActionQueue::new(),
            initialized: false,

            current_beatmap: None,
            // beatmaps: Vec::new(),
            beatmaps: HashMap::new(),
            ignore_beatmaps: HashSet::new(),
            collections: Vec::new(),
            diffs: HashMap::new(),

            played: Vec::new(),
            play_index: 0,

            filter_text: String::new(),
            unfiltered_groups: Vec::new(),
            groups: Vec::new(),
            selected_set: 0,
            selected_map: 0
        }
    }

    pub fn initialize(
        &mut self,
        sort_by: SortBy,
        group_by: GroupBy,
        mods: &Mods,
        playmode: &str,
        diff_manager: &mut impl DifficultyProvider,
    ) {
        trace!("Beatmap manager initialized");
        self.initialized = true;
        self.refresh_maps(mods, playmode, sort_by, group_by, diff_manager);
    }

    pub fn add_played(&mut self, map: Md5Hash) {
        self.played.push(map);
        self.play_index += 1;
    }
    pub fn remove_played(&mut self, index_change: usize) {
        self.played.pop();
        self.play_index -= index_change;
    }


    pub fn set_current(&mut self, hash: Md5Hash) {
        if !self.beatmaps.contains_key(&hash)
        { return }

        self.current_beatmap = Some(hash);

        // make sure we have the selected set and selected map values up to date
        for (n, i) in self.groups.iter_mut().enumerate() {
            i.selected = false;

            if let Some(j) = i.has_hash(&hash) {
                self.selected_set = n;
                self.selected_map = j;
                i.selected = true;
            }
        }
    }


    pub fn folders_to_check(settings: &engine::Settings) -> Vec<std::path::PathBuf> {
        let mut dirs_to_check = settings.external_games_folders.clone();
        dirs_to_check.push(engine::SONGS_DIR.to_owned());

        dirs_to_check.iter()
            .map(std::fs::read_dir)
            .filter_map(Result::ok)
            .flat_map(|f| f
                .filter_map(Result::ok)
                .map(|f| f.path())
            )
            .collect()
    }

    /// clear the cache and db, and do a full rescan of the songs folder
    pub fn full_refresh(&mut self, settings: &engine::Settings) {
        self.actions.push(actions::database::Action::ClearAllBeatmaps.into());

        self.beatmaps.clear();
        self.diffs.clear();
        self.initialized = false;

        let mut new_beatmaps = Vec::new();

        info!("Reading maps");
        for f in Self::folders_to_check(settings) {
            let Some(maps) = self.check_folder(
                f,
                false,
            ) else { continue };

            new_beatmaps.extend(maps);
        }

        self.initialized = true;
        if !new_beatmaps.is_empty() {
            self.actions.push(actions::database::Action::AddBeatmaps(new_beatmaps).into());
        }
    }

    /// if this doesnt handle the database entries, returns a list of new beatmaps that should be added to the database
    pub fn check_folder(
        &mut self,
        dir: impl AsRef<Path>,
        handle_database: impl Into<HandleDatabase>,
    ) -> Option<Vec<Arc<BeatmapMeta>>> {
        let handle_database = handle_database.into();
        let dir = dir.as_ref();

        if !dir.is_dir() { return None }
        let dir_files = read_dir(dir).unwrap();

        let mut ignore_paths = self.ignore_beatmaps.clone();

        // ignore existing paths
        for i in self.beatmaps.values() {
            ignore_paths.insert(i.file_path.clone());
        }

        let mut maps_to_add_to_database = Vec::new();

        for file in dir_files.filter_map(Result::ok) {
            let file = file.path();
            if file.is_dir() {
                let Some(maps) = self.check_folder(
                    &file,
                    handle_database,
                ) else { continue };

                maps_to_add_to_database.extend(maps);
                continue;
            }

            let Some(file) = file.to_str()
            else { continue };
            // info!("checking {file}");

            if engine::beatmaps::AVAILABLE_MAP_EXTENSIONS.iter().any(|e| file.ends_with(e)) {
                // check file paths first
                if ignore_paths.contains(&ArcStr::from(file)) {
                    continue
                }

                match tataku::fs::get_file_hash(file) {
                    Ok(hash) => if self.beatmaps.contains_key(&hash) {
                        continue;
                    },
                    Err(e) => {
                        error!("error getting hash for file {file}: {e}");
                        continue;
                    }
                }

                match Beatmap::load_multiple_metadata(file) {
                    Ok(maps) => {
                        for map in maps {
                            self.add_beatmap(&map, false);

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

        if handle_database.insert_into_database() {
            self.actions.push(actions::database::Action::AddBeatmaps(
                maps_to_add_to_database.clone()
            ).into());
        }

        if handle_database.return_new_maps() {
            Some(maps_to_add_to_database)
        } else {
            None
        }
    }

    pub fn add_beatmap(
        &mut self,
        beatmap: &Arc<BeatmapMeta>,
        add_to_db: bool,
    ) {
        // check if we already have this map
        if self.beatmaps.contains_key(&beatmap.beatmap_hash) {
            trace!("Map already added");

            // see if this beatmap is being added from another source
            if !self.beatmaps
                .values()
                .any(|m| m.file_path == beatmap.file_path)
            {
                // if so, add it to the ignore list
                trace!("Adding {} to the ignore list", beatmap.file_path);
                self.ignore_beatmaps.insert(beatmap.file_path.clone());
                self.actions.push(actions::database::Action::IgnoreBeatmap(
                    engine::data::IgnoredBeatmap::Path(beatmap.file_path.to_string())
                ).into());
            }

            return;
        }

        // dont have it, add it
        let new_hash = beatmap.beatmap_hash;
        self.beatmaps.insert(new_hash, beatmap.clone());

        if self.initialized {
            debug!("Adding beatmap {}", beatmap.version_string());

            #[cfg(feature="graphics")]
            self.actions.push(actions::game::GameAction::HandleEvent(
                input::TatakuEvent::MapAdded,
                Some(beatmap.beatmap_hash.to_string().into())
            ).into());
        }

        if add_to_db {
            self.actions.push(actions::database::Action::AddBeatmaps(vec![beatmap.clone()]).into());
        }

    }

    pub fn delete_beatmap(&mut self, beatmap: Md5Hash) -> bool {
        // remove beatmap from ourselves
        // self.beatmaps.retain(|b| b.beatmap_hash != beatmap);

        let Some(old_map) = self.beatmaps.remove(&beatmap)
        else { return false };

        if old_map.file_path.starts_with(engine::SONGS_DIR) {

            // delete the file
            if let Err(e) = std::fs::remove_file(&*old_map.file_path) {
                self.actions.push(Notification::new_error(
                    "Error deleting map",
                    e
                ).into());
            }
            // TODO: should check if this is the last beatmap in this folder
            // if so, delete the parent dir
        } else {
            // file is probably in an external folder, just add this file to the ignore list
            self.ignore_beatmaps.insert(old_map.file_path.clone());

            self.actions.push(actions::database::Action::IgnoreBeatmap(
                engine::data::IgnoredBeatmap::Hash(beatmap)
            ).into());
        }

        self.current_beatmap == Some(beatmap)
    }


    pub fn current_beatmap(&self) -> Option<&Arc<BeatmapMeta>> {
        self.beatmaps.get(self.current_beatmap.as_ref()?)
    }


    // getters
    pub fn all_by_sets(&self, group_by: engine::data::GroupBy) -> Vec<BeatmapGroup> {
        let mut set_map: HashMap<BeatmapGroupValue, BeatmapGroup> = HashMap::new();

        match group_by {
            tataku_engine::data::GroupBy::Set => {
                for beatmap in self.beatmaps.values() {
                    let key = format!(
                        "[{}] // {} - {}",
                        beatmap.creator,
                        beatmap.artist,
                        beatmap.title
                    );
                    let key = BeatmapGroupValue::Set(key);

                    let list = set_map
                        .entry(key.clone())
                        .or_insert_with(|| BeatmapGroup::new(key.clone()));

                    list.maps.push(beatmap.beatmap_hash);
                }
            }
            tataku_engine::data::GroupBy::Collections => {
                for i in self.collections.iter() {
                    let key = BeatmapGroupValue::Collection(i.name.clone());

                    let list = set_map
                        .entry(key.clone())
                        .or_insert_with(|| BeatmapGroup::new(key.clone()));

                    for i in i.beatmaps.iter() {
                        list.maps.push(*i);
                    }
                }
            }
        }

        set_map.into_values().collect()
    }

    pub fn has_hash(&self, hash: &Md5Hash) -> bool {
        self.beatmaps.contains_key(hash)
    }
    pub fn get_by_hash(&self, hash: &Md5Hash) -> Option<Arc<BeatmapMeta>> {
        self.beatmaps.get(hash).cloned()
    }


    pub fn random_beatmap(&self) -> Option<Md5Hash> {
        if self.beatmaps.is_empty() {
            return None;
        }

        let ind = rand::rng().random_range(0..self.beatmaps.len());
        let map = self.beatmaps.keys().nth(ind).unwrap();

        Some(*map)
    }


    pub fn next_beatmap(&self) -> Option<Md5Hash> {
        self
            .played
            .get(self.play_index + 1)
            .copied()
    }
    pub fn previous_beatmap(&self) -> Option<Md5Hash> {
        if self.play_index == 0 { return None }

        self
            .played
            .get(self.play_index - 1)
            .copied()
    }
}

impl BeatmapManager {
    pub fn refresh_maps(
        &mut self,
        current_mods: &Mods,
        playmode: &str,
        sort_by: SortBy,
        group_by: GroupBy,
        diff_manager: &mut dyn DifficultyProvider,
    ) {
        //TODO: allow grouping by not just map set
        self.unfiltered_groups = self.all_by_sets(group_by);

        self.apply_filter(current_mods, playmode, sort_by, diff_manager);
    }

    pub fn apply_filter(
        &mut self,
        mods: &Mods,
        playmode: &str,
        sort_by: SortBy,
        diff_manager: &mut dyn DifficultyProvider,
    ) {
        trace!("Applying Filter");
        self.groups.clear();

        // get filter text and split here so we arent splitting every map
        let filter_text = &self.filter_text; //values.get_string("beatmap_list.search_text").unwrap_or_default();
        let filters = filter_text
            .split(" ")
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        for group in self.unfiltered_groups.iter() {

            // let mut selected = false;
            let maps = group.maps
            .iter()
            .flat_map(|m| {
                let meta = self.beatmaps.get(m).unwrap();

                let mode = self
                    .infos
                    .get_playmode_actual(playmode, Some(meta));

                let diff = diff_manager
                    .get_diff(meta, mode, mods)
                    .unwrap_or(-1.0);

                let info = self.infos
                    .get_info(mode)
                    .ok()?;

                let diff_info = {
                    let data = GetDiffValue {
                        map: meta,
                        mods,
                        diff,
                    };

                    info.diff_values
                        .iter()
                        .map(|dv|
                            dv.format((dv.get_diff_value)(&data))
                        )
                        .collect::<Vec<_>>()
                        .join(" | ")
                };

                let entry = self.diffs
                    .entry(*m)
                    .or_default();

                entry.diff = diff;
                entry.info = diff_info.into();

                // apply filter
                for filter in filters.iter() {
                    if !meta.filter(filter, diff) {
                        return None;
                    }
                }

                Some(*m)
            })
            .collect::<Vec<_>>();

            if maps.is_empty() { continue }

            let name = group.get_name().clone();
            self.groups.push(BeatmapListGroup { maps, id: 0, name, selected: false });
        }

        self.sort(sort_by);
    }

    pub fn sort(
        &mut self,
        sort_by: SortBy,
    ) {
        let current_hash = self
            .current_beatmap
            .as_ref()
            .copied();

        // sort
        macro_rules! sort {
            ($property:tt, String) => {
                self.groups.sort_by(|a, b| self.beatmaps.get(&a.maps[0]).unwrap().$property.to_lowercase()
                    .cmp(&self.beatmaps.get(&b.maps[0]).unwrap().$property.to_lowercase()))
            };
            ($property:ident, Float) => {
                self.groups.sort_by(|a, b| self.beatmaps.get(&a.maps[0]).$property
                    .partial_cmp(self.beatmaps.get(&b.maps[0]).$property).unwrap())
            };
        }

        match sort_by {
            SortBy::Title => sort!(title, String),
            SortBy::Artist => sort!(artist, String),
            SortBy::Creator => sort!(creator, String),
            SortBy::Difficulty => {
                self.groups.sort_by(|a, b| {
                    let Some(a) = self.diffs.get(&a.maps[0])
                    else { return std::cmp::Ordering::Equal };

                    let Some(b) = self.diffs.get(&b.maps[0])
                    else { return std::cmp::Ordering::Equal };

                    a.diff.total_cmp(&b.diff)
                });
            },
        }

        let mut selected = false;
        for (n, i) in self
            .groups
            .iter_mut()
            .enumerate()
        {
            i.id = n;
            i.selected = false;

            // make sure we have the correct selected set and map number
            if !selected
            && let Some(current_hash) = &current_hash
            && let Some(j) = i.has_hash(current_hash) {
                self.selected_set = n;
                self.selected_map = j;
                selected = true;
                i.selected = true;
            }
        }

    }

    pub fn select_set(&mut self, set_num: usize) {
        trace!("Selecting set: {set_num}");

        self.selected_set = set_num;
        self.select_map(0);

        // FIXME: !!!
        // #[cfg(feature="graphics")]
        // self.actions.push(actions::Action::PerformOperation(
        //     snap_to_id(
        //     "beatmap_scroll",
        //     iced::widget::scrollable::RelativeOffset {
        //         x: 0.0,
        //         y: set_num as f32 / self.groups.len() as f32
        //     })
        // ))
    }
    pub fn next_set(&mut self) {
        self.select_set(
            (self.selected_set + 1)
                .wrapping_clamp(0, self.groups.len())
        );
    }
    pub fn prev_set(&mut self) {
        self.select_set(
            (self.selected_set - 1)
                .wrapping_clamp(0, self.groups.len())
        );
    }

    pub fn select_map(&mut self, map_num: usize)  {
        self.selected_map = map_num;

        let Some(set) = self.groups.get(self.selected_set)
        else { return };

        if let Some(map) = set.maps.get(self.selected_map) {
            self.actions.push(actions::beatmap::BeatmapAction::Set(
                *map,
                actions::beatmap::SetBeatmapOptions::default().use_preview_point(true)
            ).into());
        }
    }
    pub fn next_map(&mut self) {
        let Some(set) = self.groups.get(self.selected_set)
        else { return };

        self.select_map(
            (self.selected_map + 1).wrapping_clamp(0, set.maps.len())
        );
    }
    pub fn prev_map(&mut self) {
        let Some(set) = self.groups.get(self.selected_set)
        else { return };

        self.select_map(
            (self.selected_map - 1).wrapping_clamp(0, set.maps.len())
        );
    }
}
