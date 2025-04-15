use crate::prelude::*;

#[derive(Default)]
pub struct CheckBeatmapFoldersTask {
    state: TatakuTaskState,
    existing_paths: HashSet<String>,
    folders: Vec<String>
}

#[async_trait]
impl TatakuTask for CheckBeatmapFoldersTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Check Beatmap Folders") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    async fn run(
        &mut self, 
        values: &mut dyn Reflect, 
        _state: &TaskGameState,
        _actions: &mut ActionQueue
    ) {
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

            self.state = TatakuTaskState::Running;
            debug!("Got existing maps");
            return;
        }

        trace!("Loading from the disk");
        if let Some(folder) = self.folders.pop() {
            let manager = values.reflect_get_mut::<BeatmapManager>("beatmap_manager").expect("nope");

            manager.check_folder(folder, true).await;
            return
        }

        debug!("Done checking maps folders");
        self.state = TatakuTaskState::Complete;
    }
}
