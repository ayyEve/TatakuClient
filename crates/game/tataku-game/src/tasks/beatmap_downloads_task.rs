use crate::prelude::*;

use engine::{
    actions,
    game::task::*,
    beatmaps::{
        Beatmap,
        BeatmapMeta,
        AVAILABLE_MAP_EXTENSIONS,
    },
};

const DOWNLOAD_CHECK_INTERVAL:u64 = 10_000;
const IGNORED_EXTENSIONS: &[&str] = &[
    ".wav",
    ".ogg",
    ".mp3",
    ".osk",
    ".osb",
    ".png",
    ".jpg",
];

#[derive(Default2)]
pub struct BeatmapDownloadsCheckTask {
    last_check: u64,

    maps_to_add: Vec<Arc<BeatmapMeta>>,

    #[default(Box::new(Vec::new().into_iter()))]
    files: Box<dyn Iterator<Item = PathBuf> + Send + Sync>,
}

impl TatakuTask for BeatmapDownloadsCheckTask {
    fn get_name(&self) -> CowStr { Cow::Borrowed("Beatmap Download Check") }
    fn get_type(&self) -> TatakuTaskType { TatakuTaskType::Continuous }
    fn get_state(&self) -> TatakuTaskState { TatakuTaskState::Running } // no real point in saying we arent running, since we run for one update every ~10s

    fn run(&mut self, shell: &mut TaskShell) {
        // dont continue if we're ingame
        if shell.ingame { return }

        // check if we need to add any beatmaps
        if let Some(map) = self.maps_to_add.pop() {
            info!("Adding map {}", map.version_string());
            shell.actions.push(actions::beatmap::BeatmapAction::AddBeatmap { 
                map, 
                add_to_db: true 
            }.into());
            return 
        }

        // check if we're processing any files
        if let Some(file) = self.files.next() {
            info!("Checking file {file:?}");
            let Some(file) = file.to_str() 
            else { return };
            
            info!("File ok {file}");
            
            if AVAILABLE_MAP_EXTENSIONS.iter().any(|e| file.ends_with(e)) {
                match Beatmap::load_multiple_metadata(file) {
                    Ok(maps) => {
                        info!("map loaded ok");
                        self.maps_to_add.extend(maps);
                    }

                    Err(e) 
                        => error!("error loading beatmap '{file}': {e}"),
                }
            } else {
                if IGNORED_EXTENSIONS.iter().any(|i| file.ends_with(i)) {
                    return;
                }

                warn!("Map ext not found! {file}");
            }

            return;
        }

        // only check the folder every X seconds
        if shell.game_time - self.last_check < DOWNLOAD_CHECK_INTERVAL { return }

        // get all files in the downloads dir
        let dir = std::fs::read_dir(engine::DOWNLOADS_DIR)
            .unwrap()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        if dir.is_empty() { return }

        // extract them to the songs dir
        let mut folders = Vec::new();

        // TODO: this is kinda shit
        for i in dir {
            let path = i.path();
            let Some(ext) = path.extension() else { continue };
            if ext == ".osk" {
                if let Ok(path) = engine::io::Zip::extract_single(
                    i.path(), 
                    engine::SKINS_FOLDER, 
                    true, 
                    engine::io::ArchiveDelete::Always
                ) {
                    folders.push(path);
                }
            } else if let Ok(path) = engine::io::Zip::extract_single(
                i.path(), 
                engine::SONGS_DIR, 
                true, 
                engine::io::ArchiveDelete::Always
            ) {
                folders.push(path);
            }
        }
        // let folders = Zip::extract_all(DOWNLOADS_DIR, SONGS_DIR, ArchiveDelete::Always).await;
        // info!("checking folders {folders:#?}");

        // add extracted maps
        self.files = Box::new(
            folders
            .into_iter()
            .map(|f| Path::new(&f).to_path_buf()) // into path
            .filter(|p| p.exists() && p.is_dir()) // make sure exists and is a directory
            .filter_map(|p| std::fs::read_dir(p)
                .map_err(|e| 
                    error!("Error reading extracted path: {e:?}")
                ).ok()
            ) // read files in the path, make sure the read was okay
            
            .flat_map(|f| f
                // files in the folder
                .filter_map(|f| 
                    f.map_err(|e| 
                        error!("Error reading extracted file: {e:?}")
                    ).ok()
                ) // make sure file read is okay
                .map(|f| f.path()) // map to path
            )
        );
    }
}
