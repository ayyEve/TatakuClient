use crate::prelude::*;

#[derive(Default)]
pub struct BeatmapListComponent {
    actions: ActionQueue,

    /// cache of groups before we filter them, saved from rebuilding this list every filter update
    unfiltered_groups: Vec<Vec<Arc<BeatmapMeta>>>,
    filtered_groups: Vec<BeatmapGroup>,

    /// variable name for the filter
    filter_var: Option<String>,
    last_filter: String,

    /// TODO!!
    sort_method: SortBy,

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
        debug!("Refreshing maps");
        //TODO: allow grouping by not just map set
        self.unfiltered_groups = BEATMAP_MANAGER.read().await.all_by_sets(GroupBy::Title);

        // let diff_calc_helper = beatmap_manager.on_diffcalc_completed.1.clone();

        self.apply_filter(values).await;

        // update diffs
        // let mode_clone = self.mode.clone();
        // tokio::spawn(async {
        //     BEATMAP_MANAGER.write().await.update_diffs(mode_clone, &*ModManager::get().await);
        // });
    }
    
    pub async fn apply_filter(&mut self, values: &mut ShuntingYardValues) {
        debug!("Applying Filter");
        self.filtered_groups.clear();
        let current_hash = Self::current_hash();
        // let current_beatmap = CurrentBeatmapHelper::new().0.clone();

        // self.beatmap_scroll.clear();
        let filter_text = self.last_filter.to_ascii_lowercase();

        let mods = ModManagerHelper::new(); //self.mods.clone();
        let mode = CurrentPlaymodeHelper::new().0.clone(); //self.get_playmode(); //Arc::new(self.mode.clone());

        // used to select the current map in the list
        // let current_hash = current_beatmap.map(|m| m.beatmap_hash).unwrap_or_default();
        // let mut n = 0;

        // let mut selected_set = 0;
        // let mut selected_map = 0;
        let mut filtered_groups = Vec::new();

        for maps in self.unfiltered_groups.iter() {
            let mut maps:Vec<BeatmapMetaWithDiff> = maps.iter().enumerate().map(|(_n2, m)| {
                let mode = m.check_mode_override(mode.clone());
                let diff = get_diff(&m, &mode, &mods);
                //if diff.is_none() { modes_needing_diffcalc.insert(mode); }
                // if m.comp_hash(current_hash) {
                //     selected_set = n;
                //     selected_map = n2;
                // }

                BeatmapMetaWithDiff::new(m.clone(), diff)
            }).collect();

            if !filter_text.is_empty() {
                let filters = filter_text.split(" ");

                for filter in filters {
                    maps.retain(|bm| bm.filter(&filter));
                }

                if maps.is_empty() { continue }
            }

            let meta = &maps[0];
            let name = format!("{} // {} - {}", meta.creator, meta.artist, meta.title);
            filtered_groups.push(BeatmapGroup {
                maps,
                number: 0,
                name,
            });
            // n += 1;

            // let n = self.visible_sets.len();
            // let meta = &maps[0];
            // let display_text = format!("{} // {} - {}", meta.creator, meta.artist, meta.title);
            // // let mut i = BeatmapsetItem::new(maps, display_text).await;
            // let mut set_item = BeatmapSetComponent::new(display_text, n, maps).await;
            // if let Some(map_num) = set_item.check_selected(current_hash) {
            //     selected_set = n;
            //     selected_map = map_num;
            // }
            // self.visible_sets.push(set_item);
        }

        // // make sure the correct set and map are selected
        // // this will also scroll to the selected set
        // self.select_set(selected_set);
        // self.select_map(selected_map);

        // sort
        macro_rules! sort {
            ($property:tt, String) => {
                filtered_groups.sort_by(|a, b| a.maps[0].$property.to_lowercase().cmp(&b.maps[0].$property.to_lowercase()))
            };
            ($property:ident, Float) => {
                filtered_groups.sort_by(|a, b| a.maps[0].$property.partial_cmp(&b.maps[0].$property).unwrap())
            }
        }

        match self.sort_method {
            SortBy::Title => sort!(title, String),
            SortBy::Artist => sort!(artist, String),
            SortBy::Creator => sort!(creator, String),
            SortBy::Difficulty => sort!(diff, Float),
        }

        // // we need to renumber because the sort changes the numbers
        // self.filtered_groups
        //     .iter_mut()
        //     .enumerate()
        //     .for_each(|(n, s)|s.num = n);

        // self.visible_sets = full_list;
        // for i in full_list { 
        //     self.show_maps.push(i);
        //     // self.beatmap_scroll.add_item(i) 
        // }
        // self.beatmap_scroll.scroll_to_selection();
            
        let mut selected = None;
        self.filtered_groups = filtered_groups;
        for (n, i) in self.filtered_groups.iter_mut().enumerate() {
            i.number = n;
            
            if selected.is_none() {
                if let Some((n2, _)) = i.maps.iter().enumerate().find(|(_, m)| m.comp_hash(current_hash)) {
                    selected = Some((n, n2))
                }
            }
        }

        if let Some((set, map)) = selected {
            self.select_set(set, values);
            self.select_map(map, values);
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
        // debug!("values: {values:#?}");
    }
    fn current_hash() -> Md5Hash {
        CurrentBeatmapHelper::new().0.as_ref().map(|m| m.beatmap_hash).unwrap_or_default()
    }
    
    // menu event helpers
    fn select_set(&mut self, set_num: usize, values: &mut ShuntingYardValues) {
        debug!("selecting set: {set_num}");
        // self.filtered_groups.get_mut(self.selected_set).ok_do_mut(|set| set.selected = false);
        // self.filtered_groups.get_mut(set_num).ok_do_mut(|set| set.selected = true);
        
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
        // self.updates += 1;
        self.selected_map = map_num;

        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        if let Some(map) = set.maps.get(self.selected_map) {
            self.actions.push(BeatmapMenuAction::Set(map.meta.clone(), true));
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
        if self.unfiltered_groups.is_empty() {
            self.refresh_maps(values).await;
        }

        // check the filter
        if let Some(Ok(filter)) = self.filter_var.as_ref().map(|f| values.get_string(f)) {
            if self.last_filter != filter {
                self.apply_filter(values).await;

                self.last_filter = filter;
            }
        }

    }
    
    async fn handle_message(&mut self, message: &Message, values: &mut ShuntingYardValues) -> ActionQueue { 
        
        if let MessageTag::String(tag) = &message.tag {
            match &**tag {
                "beatmap_list.set_beatmap" => {
                    let Some(hash) = message.message_type.as_text_ref() else { panic!("invalid type for beatmap hash: {:?}", message.message_type) };
                    let Ok(hash) = Md5Hash::try_from(hash) else { panic!("invalid hash string for beatmap hash") };
                    self.actions.push(BeatmapMenuAction::SetFromHash(hash, true));
                    self.update_values(values, hash);
                }

                "beatmap_list.set_set" => {
                    let set_num = match &message.message_type {
                        MessageType::Number(n) => *n,
                        MessageType::Value(CustomElementValue::U64(n)) => (*n) as usize,
                        other => {
                            error!("invalid type for set number: {other:?}");
                            return ActionQueue::default();
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

        std::mem::take(&mut self.actions)
    }
}

pub struct BeatmapGroup {
    pub number: usize,
    pub name: String,
    pub maps: Vec<BeatmapMetaWithDiff>,
}
impl BeatmapGroup {
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