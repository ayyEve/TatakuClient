
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

    pub async fn refresh_maps(&mut self, values: &mut ValueCollection, ) {
        trace!("Refreshing maps");
        //TODO: allow grouping by not just map set
        self.unfiltered_groups = BEATMAP_MANAGER.read().await.all_by_sets(GroupBy::Set);

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

    fn sort(&mut self, values: &mut ValueCollection) {
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



    fn update_values(&mut self, values: &mut ValueCollection, current_hash: Md5Hash) {
        let filtered_groups = CustomElementValue::List(
            self.filtered_groups
                .iter()
                .map(|group| group.into_map(current_hash)).collect()
        );

        values.set("beatmap_list.groups", filtered_groups);
    }

    // menu event helpers
    fn select_set(&mut self, set_num: usize, values: &mut ValueCollection) {
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
    fn next_set(&mut self, values: &mut ValueCollection) {
        self.select_set(self.selected_set.wrapping_add_1(self.filtered_groups.len()), values)
    }
    fn prev_set(&mut self, values: &mut ValueCollection) {
        self.select_set(self.selected_set.wrapping_sub_1(self.filtered_groups.len()), values)
    }

    fn select_map(&mut self, map_num: usize, values: &mut ValueCollection)  {
        self.selected_map = map_num;

        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        if let Some(map) = set.maps.get(self.selected_map) {
            self.actions.push(BeatmapAction::Set(map.meta.clone(), true, false));
            self.update_values(values, map.beatmap_hash);
        }

    }
    fn next_map(&mut self, values: &mut ValueCollection) {
        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_add_1(set.maps.len()), values)
    }
    fn prev_map(&mut self, values: &mut ValueCollection) {
        let Some(set) = self.filtered_groups.get(self.selected_set) else { return };
        self.select_map(self.selected_map.wrapping_sub_1(set.maps.len()), values)
    }

}

#[async_trait]
impl Widgetable for BeatmapListComponent {
    async fn update(&mut self, values: &mut ValueCollection, _actions: &mut ActionQueue) {
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
        if let Ok(sort_by) = values.try_get::<SortBy>("settings.sort_by") {
            if self.sort_by != sort_by {
                trace!("sort_by changed, re-sorting");
                self.sort_by = sort_by;
                self.sort(values);
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
    
    async fn handle_message(&mut self, message: &Message, values: &mut ValueCollection) -> Vec<TatakuAction> { 
        
        if let MessageTag::String(tag) = &message.tag {
            match &**tag {
                "beatmap_list.set_beatmap" => {
                    let Some(hash) = message.message_type.as_text_ref() else { panic!("invalid type for beatmap hash: {:?}", message.message_type) };
                    let Ok(hash) = Md5Hash::try_from(hash) else { panic!("invalid hash string for beatmap hash") };
                    self.actions.push(BeatmapAction::SetFromHash(hash, true, false));
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
