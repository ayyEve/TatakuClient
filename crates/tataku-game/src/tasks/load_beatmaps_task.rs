use crate::prelude::*;

pub struct LoadBeatmapsTask {
    state: TatakuTaskState,
    
    status_index: usize,

    /// list of ignored file paths
    ignored_list: Vec<String>,

    /// list of maps loaded from the database
    existing_maps: Vec<Arc<BeatmapMeta>>,
}
impl LoadBeatmapsTask {
    pub fn new(status_index: usize) -> Self {
        Self {
            state: TatakuTaskState::NotStarted,
            status_index,
            ignored_list: Vec::new(),
            existing_maps: Vec::new(),
        }
    }
}

impl TatakuTask for LoadBeatmapsTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Load Beatmap Task") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(
        &mut self, 
        values: &mut dyn Reflect, 
        _state: &TaskGameState, 
        actions: &mut ActionQueue,
    ) {
        let statuses = values
            .reflect_get_mut::<Vec<LoadingStatus>>("game.loading_statuses")
            .unwrap();
        let status = &mut statuses[self.status_index];

        // if we havent started yet, initialize our values
        if self.state == TatakuTaskState::NotStarted {
            self.ignored_list = Database::get_all_ignored();
            self.existing_maps = Database::get_all_beatmaps();
            status.item_count = self.existing_maps.len();

            self.state = TatakuTaskState::Running;
            debug!("Got existing maps");
            return;
        }

        // load all maps from the database
        if let Some(map) = self.existing_maps.pop() {
            // trace!("Adding map {}", map.beatmap_hash);

            // make sure the beatmap exists before adding it
            if !Io::exists(&map.file_path) {
                warn!("Beatmap exists in db but not in fs: {}", map.file_path);
            } else {
                actions.push(BeatmapAction::AddBeatmap { 
                    map, 
                    add_to_db: false 
                });
            }
            status.items_complete += 1;
            return;
        }

        debug!("Done adding maps");
        actions.push(BeatmapAction::InitializeManager);
        status.complete = true;
        self.state = TatakuTaskState::Complete;

        // add a task to check the beatmaps folder for new maps
        actions.push(TaskAction::AddTask(Box::new(CheckBeatmapFoldersTask::default())));
    }
}
