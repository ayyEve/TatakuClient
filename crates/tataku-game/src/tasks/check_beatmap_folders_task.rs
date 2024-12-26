use crate::prelude::*;

pub struct CheckBeatmapFoldersTask {
    state: TatakuTaskState,
    // status: Arc<RwLock<LoadingStatus>>,


    existing_paths: HashSet<String>,

    folders: Vec<String>
}
impl CheckBeatmapFoldersTask {
    pub fn new(
        // status: Arc<RwLock<LoadingStatus>>
    ) -> Self {
        Self {
            state: TatakuTaskState::NotStarted,
            // status,

            existing_paths: HashSet::new(),
            folders: Vec::new()
        }
    }
}

#[async_trait]
impl TatakuTask for CheckBeatmapFoldersTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Check Beatmap Folders") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(&mut self, values: &mut dyn Reflect, _state: &TaskGameState, _actions: &mut ActionQueue) {

        // if we havent started yet, initialize our values
        if self.state == TatakuTaskState::NotStarted {
            let beatmap_manager = values.reflect_get::<BeatmapManager>("beatmaps").expect("nope");
            let settings = values.reflect_get("settings").expect("nope");

            // get existing dirs
            for i in beatmap_manager.beatmaps.iter() {
                if let Some(parent) = Path::new(&*i.file_path).parent() {
                    self.existing_paths.insert(parent.to_string_lossy().to_string());
                }
            }
            
            // filter out folders that already exist
            let folders = BeatmapManager::folders_to_check(&settings);
            self.folders = folders
                .into_iter()
                .map(|f| f.to_string_lossy().to_string())
                .filter(|f| !self.existing_paths.contains(f))
                .collect();


            // {
            //     let mut lock = self.status.write();
            //     lock.items_complete = 0;
            //     lock.item_count = self.folders.len();
            //     lock.custom_message = "Checking folders...".to_owned();
            // }
            
            self.state = TatakuTaskState::Running;
            debug!("Got existing maps");
            return;
        }

        trace!("Loading from the disk");
        if let Some(folder) = self.folders.pop() {
            let manager = values.reflect_get_mut::<BeatmapManager>("beatmap_manager").expect("nope");
        
            manager.check_folder(folder, true).await;
            // self.status.write().items_complete += 1;
            return
        }

        // let nlen = manager.beatmaps.len();
        // debug!("loaded {nlen} beatmaps ({} new)", nlen - existing_len);


        // otherwise, we're done!
        debug!("Done checking maps folders");
        // self.status.write().complete = true;
        self.state = TatakuTaskState::Complete;
    }
}
