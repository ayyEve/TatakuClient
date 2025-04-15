use crate::prelude::*;

pub struct LoadBeatmapsTask {
    state: TatakuTaskState,

    status: Arc<RwLock<LoadingStatus>>,

    /// list of ignored file paths
    ignored_list: Vec<String>,

    /// list of maps loaded from the database
    existing_maps: Vec<Arc<BeatmapMeta>>,
}
impl LoadBeatmapsTask {
    pub fn new(status: Arc<RwLock<LoadingStatus>>) -> Self {
        Self {
            state: TatakuTaskState::NotStarted,
            status,
            ignored_list: Vec::new(),
            existing_maps: Vec::new(),
        }
    }
}

#[async_trait]
impl TatakuTask for LoadBeatmapsTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Load Beatmap Task") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(
        &mut self, 
        _values: &mut dyn Reflect, 
        _state: &TaskGameState, 
        actions: &mut ActionQueue
    ) {

        // if we havent started yet, initialize our values
        if self.state == TatakuTaskState::NotStarted {
            self.ignored_list = Database::get_all_ignored().await;
            self.existing_maps = Database::get_all_beatmaps().await;
            // self.existing_maps.reverse(); // because they're added in reverse order later, but it doesnt really matter

            self.state = TatakuTaskState::Running;
            debug!("Got existing maps");
            return;
        }

        // load all maps from the database
        if let Some(map) = self.existing_maps.pop() {
            trace!("Adding map {}", map.beatmap_hash);

            // make sure the beatmap exists before adding it
            if !std::path::Path::new(&map.file_path).exists() {
                warn!("Beatmap exists in db but not in fs: {}", map.file_path);
            } else {
                actions.push(BeatmapAction::AddBeatmap { map, add_to_db: false });
            }

            // if that was the last map, tell the beatmap manager it has been initialized
            if self.existing_maps.is_empty() {
                debug!("All existing maps loaded");
                // actions.push(BeatmapAction::InitializeManager);
            }

            return;
        }

        debug!("Done adding maps");
        actions.push(BeatmapAction::InitializeManager);
        self.status.write().complete = true;
        self.state = TatakuTaskState::Complete;

        // add a task to check the beatmaps folder for new maps
        actions.push(TaskAction::AddTask(Box::new(CheckBeatmapFoldersTask::default())));
    }
}
