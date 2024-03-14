/**
 * Beatmap List Component
 * Provides a list of grouped beatmaps
 * 
 * Parameters:
 * - filter_var: Option<String> -> What variable to use to filter maps
 * 
 * Values:
 *  - beatmap_list.groups: List<BeatmapGroup> -> The list of (filtered) groups
 * 
 * Types:
 * - BeatmapGroup:
 *      - maps: List<Beatmap> -> The list of maps in the group
 *      - number: Number -> The number id for this set
 * 
 * Behaviors:
 *  - beatmap_list.set_beatmap(hash: String) -> Set the current beatmap to the provided hash
 *  - beatmap_list.set_set(set_num: Number) -> Set the current set to the provided set number
 *  - beatmap_list.next_map -> Set the current beatmap to the next map in the set
 *  - beatmap_list.prev_map -> Set the current beatmap to the previous map in the set
 * 
 *  - beatmap_list.next_set -> Set current set to the next set in the list
 *  - beatmap_list.prev_set -> Set current set to the previous set in the list
 */

use crate::prelude::*;

#[derive(Default)]
pub struct BeatmapListComponent {
    actions: ActionQueue,

    /// cache of groups before we filter them, saved from rebuilding this list every filter update
    unfiltered_groups: Vec<BeatmapGroup>,
    filtered_groups: Vec<BeatmapListGroup>,

    /// variable name for the filter
    filter_var: Option<String>,
    filter: String,

    sort_by: SortBy,
    group_by: GroupBy,

    selected_set: usize,
    selected_map: usize,
}
impl BeatmapListComponent {
    pub fn new(filter: Option<String>) -> Self {
        Self {
            filter_var: filter,

            ..Self::default()
        }
    }

    pub async fn refresh_maps(&mut self, values: &mut ShuntingYardValues) {
        trace!("Refreshing maps");
        //TODO: allow grouping by not just map set
        self.unfiltered_groups = BEATMAP_MANAGER.read().await.all_by_sets(GroupBy::Set);

        self.apply_filter(values).await;
    }
    
    pub async fn apply_filter(&mut self, values: &mut ShuntingYardValues) {
        trace!("Applying Filter");
        self.filtered_groups.clear();
        let current_hash = CurrentBeatmapHelper::new().0.as_ref().map(|m| m.beatmap_hash).unwrap_or_default();
        
        // get filter text and split here so we arent splitting every map
        let filter_text = self.filter.to_ascii_lowercase();
        let filters = filter_text.split(" ").filter(|s| !s.is_empty()).collect::<Vec<_>>();

        let mods = ModManagerHelper::new(); //self.mods.clone();
        let mode = CurrentPlaymodeHelper::new().0.clone(); //self.get_playmode(); //Arc::new(self.mode.clone());

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


        // sort
        macro_rules! sort {
            ($property:tt, String) => {
                self.filtered_groups.sort_by(|a, b| a.maps[0].$property.to_lowercase().cmp(&b.maps[0].$property.to_lowercase()))
            };
            ($property:ident, Float) => {
                self.filtered_groups.sort_by(|a, b| a.maps[0].$property.partial_cmp(&b.maps[0].$property).unwrap())
            }
        }

        match self.sort_by {
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



    fn update_values(&mut self, values: &mut ShuntingYardValues, current_hash: Md5Hash) {
        let filtered_groups = CustomElementValue::List(
            self.filtered_groups
                .iter()
                .map(|group| group.into_map(current_hash)).collect()
        );

        values.set("beatmap_list.groups", filtered_groups);
    }

    // menu event helpers
    fn select_set(&mut self, set_num: usize, values: &mut ShuntingYardValues) {
        debug!("selecting set: {set_num}");
        
        self.selected_set = set_num;
        self.select_map(0, values);

        self.actions.push(MenuAction::PerformOperation(
            snap_to_id(
            "beatmap_scroll", 
            iced::widget::scrollable::RelativeOffset { 
                x: 0.0,
                y: set_num as f32 / self.filtered_groups.len() as f32
            })
        ))
    }
    fn next_set(&mut self, values: &mut ShuntingYardValues) {
        self.select_set(self.selected_set.wrapping_add_1(self.filtered_groups.len()), values)
    }
    fn prev_set(&mut self, values: &mut ShuntingYardValues) {
        self.select_set(self.selected_set.wrapping_sub_1(self.filtered_groups.len()), values)
    }

    fn select_map(&mut self, map_num: usize, values: &mut ShuntingYardValues)  {
        self.selected_map = map_num;

        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        if let Some(map) = set.maps.get(self.selected_map) {
            self.actions.push(BeatmapMenuAction::Set(map.meta.clone(), true, false));
            self.update_values(values, map.beatmap_hash);
        }

    }
    fn next_map(&mut self, values: &mut ShuntingYardValues) {
        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_add_1(set.maps.len()), values)
    }
    fn prev_map(&mut self, values: &mut ShuntingYardValues) {
        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_sub_1(set.maps.len()), values)
    }

}

#[async_trait]
impl Widgetable for BeatmapListComponent {
    async fn update(&mut self, values: &mut ShuntingYardValues, _actions: &mut ActionQueue) {
        // Make sure we set the values initially
        if self.unfiltered_groups.is_empty() {
            self.refresh_maps(values).await;
        }

        // check the filter
        if let Some(Ok(filter)) = self.filter_var.as_ref().map(|f| values.get_string(f)) {
            if self.filter != filter {
                debug!("filter changed, filtering maps");
                self.filter = filter;
                self.apply_filter(values).await;
            }
        }

        // check sort_by 
        if let Ok(Ok(sort_by)) = values.get_raw("global.sort_by").map(SortBy::try_from) {
            if self.sort_by != sort_by {
                trace!("sort_by changed, filtering maps");
                self.sort_by = sort_by;
                self.apply_filter(values).await;
            }
        }

        // check group_by 
        if let Ok(Ok(group_by)) = values.get_raw("global.group_by").map(GroupBy::try_from) {
            if self.group_by != group_by {
                trace!("group_by changed, reloading maps");
                self.group_by = group_by;
                self.refresh_maps(values).await;
            }
        }
    }
    
    async fn handle_message(&mut self, message: &Message, values: &mut ShuntingYardValues) -> Vec<MenuAction> { 
        
        if let MessageTag::String(tag) = &message.tag {
            match &**tag {
                "beatmap_list.set_beatmap" => {
                    let Some(hash) = message.message_type.as_text_ref() else { panic!("invalid type for beatmap hash: {:?}", message.message_type) };
                    let Ok(hash) = Md5Hash::try_from(hash) else { panic!("invalid hash string for beatmap hash") };
                    self.actions.push(BeatmapMenuAction::SetFromHash(hash, true, false));
                    self.update_values(values, hash);
                }

                "beatmap_list.set_set" => {
                    let set_num = match &message.message_type {
                        MessageType::Number(n) => *n,
                        MessageType::Value(CustomElementValue::U64(n)) => (*n) as usize,
                        other => {
                            error!("invalid type for set number: {other:?}");
                            return Vec::new();
                        }
                    };
                    self.select_set(set_num, values);
                }
                
                "beatmap_list.prev_map" => self.prev_map(values),
                "beatmap_list.next_map" => self.next_map(values),
                "beatmap_list.prev_set" => self.prev_set(values),
                "beatmap_list.next_set" => self.next_set(values),
                
                _ => {}
            }
        }

        self.actions.take()
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

            let mut map = CustomElementMapHelper::default();
            map.set("artist", &beatmap.artist);
            map.set("title", &beatmap.title);
            map.set("creator", &beatmap.creator);
            map.set("version", &beatmap.version);
            map.set("playmode", &beatmap.mode);
            map.set("game", format!("{:?}", beatmap.beatmap_type));
            map.set("diff_rating", beatmap.diff.unwrap_or_default());
            map.set("hash", &beatmap.beatmap_hash.to_string());
            map.set("audio_path", &beatmap.audio_filename);
            map.set("preview_time", beatmap.audio_preview);
            map.set("is_selected", map_is_selected);
            map.set("display_mode", gamemode_display_name(&beatmap.mode).to_owned());
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
