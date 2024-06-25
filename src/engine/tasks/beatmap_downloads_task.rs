use crate::prelude::*;

const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;


pub struct BeatmapDownloadsCheckTask {
    last_check: u64,

    maps_to_add: Vec<Arc<BeatmapMeta>>,

    files: Box<dyn Iterator<Item = PathBuf> + Send + Sync>,
}
impl BeatmapDownloadsCheckTask {
    pub fn new() -> Self {
        Self {
            last_check: 0,

            maps_to_add: Vec::new(),
            files: Box::new(Vec::new().into_iter()),
        }
    }
}

#[async_trait]
impl TatakuTask for BeatmapDownloadsCheckTask {
    fn get_name(&self) -> Cow<'static, str> { Cow::Borrowed("Beatmap Download Check") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Continuous }
    fn get_state(&self) -> TatakuTaskState { TatakuTaskState::Running } // no real point in saying we arent running, since we run for one update every ~10s

    async fn run(&mut self, _values: &mut ValueCollection, state: &TaskGameState, actions: &mut ActionQueue) {
        // dont continue if we're ingame
        if state.ingame { return }

        // check if we need to add any beatmaps
        if let Some(map) = self.maps_to_add.pop() {
            actions.push(BeatmapAction::AddBeatmap { map, add_to_db: true });
            return 
        }

        // check if we're processing any files
        if let Some(file) = self.files.next() {
            let Some(file) = file.to_str() else { return };
            
            if AVAILABLE_MAP_EXTENSIONS.iter().find(|e| file.ends_with(**e)).is_some() {
                // // check file paths first
                // if ignore_paths.contains(file) {
                //     continue
                // }

                // match Io::get_file_hash(file) {
                //     Ok(hash) => if self.beatmaps_by_hash.contains_key(&hash) { continue },
                //     Err(e) => {
                //         error!("error getting hash for file {file}: {e}");
                //         continue;
                //     }
                // }

                match Beatmap::load_multiple_metadata(file) {
                    Ok(maps) => self.maps_to_add = maps,
                    Err(e) => error!("error loading beatmap '{file}': {e}"),
                }
            }

            return;
        }

        // only check the folder every X seconds
        if state.game_time - self.last_check < DOWNLOAD_CHECK_INTERVAL { return }

        // get all files in the downloads dir
        if std::fs::read_dir(DOWNLOADS_DIR).unwrap().count() == 0 { return }

        // extract them to the songs dir
        // TODO: this is kinda shit
        let folders = Zip::extract_all(DOWNLOADS_DIR, SONGS_DIR, ArchiveDelete::Always).await;
        // info!("checking folders {folders:#?}");

        // add extracted maps
        self.files = Box::new(
            folders
            .into_iter()
            .map(|f| Path::new(&f).to_path_buf()) // into path
            .filter(|p| p.exists() && p.is_dir()) // make sure exists and is a directory
            .filter_map(|p| std::fs::read_dir(p).log_error_message("Error reading extracted path").ok()) // read files in the path, make sure the read was okay
            .map(|f| f
                // files in the folder
                .filter_map(|f| f.log_error_message("Error reading extracted file").ok()) // make sure file read is okay
                .map(|f| f.path()) // map to path
            )
            .flatten() // flatten all dirs into one iter
        );
    }
}
