use crate::prelude::*;

use engine::{
    actions,
    game::task::*,
    beatmaps::BeatmapMeta,
};

pub struct LoadBeatmapsTask {
    state: TatakuTaskState,
    
    status_index: usize,

    /// list of ignored file paths
    ignored_list: Vec<engine::data::IgnoredBeatmap>,

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
    fn get_name(&self) -> CowStr { Cow::Borrowed("Load Beatmap Task") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Once }
    fn get_state(&self) -> TatakuTaskState { self.state }

    fn run(&mut self, shell: &mut TaskShell) {
        let values = ValueCollection::from_reflect_mut(shell.values);
        let statuses = &mut values.game.loading_statuses;
        let status = &mut statuses[self.status_index];

        // if we havent started yet, initialize our values
        if self.state == TatakuTaskState::NotStarted {
            self.ignored_list = shell.database.get_ignored_beatmaps().unwrap_or_default();
            self.existing_maps = shell.database.get_beatmaps().unwrap_or_default();
            status.item_count = self.existing_maps.len();

            self.state = TatakuTaskState::Running;
            debug!("Got existing maps");
            return;
        }

        // load all maps from the database
        if let Some(map) = self.existing_maps.pop() {
            // trace!("Adding map {}", map.beatmap_hash);

            // make sure the beatmap exists before adding it
            if !tataku::fs::exists(&*map.file_path) {
                warn!("Beatmap exists in db but not in fs: {}", map.file_path);
            } else {
                shell.actions.push(actions::beatmap::BeatmapAction::AddBeatmap { 
                    map, 
                    add_to_db: false,
                }.into());
            }
            status.items_complete += 1;
            return;
        }

        debug!("Done adding maps");
        shell.actions.push(actions::beatmap::BeatmapAction::InitializeManager.into());
        status.complete = true;
        self.state = TatakuTaskState::Complete;

        // add a task to check the beatmaps folder for new maps
        shell.actions.push(actions::task::TaskAction::AddTask(
            Box::new(CheckBeatmapFoldersTask::default())
        ).into());
    }
}
