use crate::prelude::*;
/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub complete: bool,
    status: Arc<Mutex<LoadingStatus>>,
    window_size: Arc<WindowSize>,
}

impl LoadingMenu {
    pub async fn new() -> Self {
        Self {
            complete: false,
            status: Arc::new(Mutex::new(LoadingStatus::new())),
            
            window_size: WindowSize::get(),
        }
    }
    pub async fn load(&mut self) {
        let status = self.status.clone();
        
        tokio::spawn(async move {
            let status = status.clone();

            // load database
            Self::load_database(status.clone()).await;

            // preload audio 
            Self::load_audio(status.clone()).await;

            // load beatmaps
            Self::load_beatmaps(status.clone()).await;

            status.lock().await.stage = LoadingStage::Done;
        });
    }

    // loaders
    async fn load_database(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().await.stage = LoadingStage::Database;
        // let _ = crate::databases::DATABASE.lock();
    }

    async fn load_audio(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().await.stage = LoadingStage::Audio;
        // get a value from the hash, will do the lazy_static stuff and populate
        // if let Ok(a) = Audio::play_preloaded("don") {
        //     a.stop();
        // }
    }

    async fn load_beatmaps(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().await.stage = LoadingStage::Beatmaps;
        // set the count and reset the counter
        status.lock().await.loading_count = 0;
        status.lock().await.loading_done = 0;


        let ignored = Database::get_all_ignored().await;

        {
            let existing_maps = Database::get_all_beatmaps().await;
            trace!("loading {} from the db", existing_maps.len());
            
            status.lock().await.loading_count = existing_maps.len();
            // load from db
            let mut lock = BEATMAP_MANAGER.write().await;
            lock.ignore_beatmaps = ignored.into_iter().collect();

            for meta in existing_maps {
                // verify the map exists
                if !std::path::Path::new(&*meta.file_path).exists() {
                    // trace!("beatmap exists in db but not in songs folder: {}", meta.file_path);
                    continue
                }

                lock.add_beatmap(&meta);
                status.lock().await.loading_done += 1;
            }
        }
        
        // look through the songs folder to make sure everything is already added
        BEATMAP_MANAGER.write().await.initialized = true; // these are new maps

        tokio::spawn(async move {

            // get existing dirs
            let mut existing_paths = HashSet::new();
            for i in BEATMAP_MANAGER.read().await.beatmaps.iter() {
                if let Some(parent) = Path::new(&*i.file_path).parent() {
                    existing_paths.insert(parent.to_string_lossy().to_string());
                }
            }
                
            // filter out folders that already exist
            let folders = BeatmapManager::folders_to_check().await;
            let folders:Vec<String> = folders.into_iter().map(|f|f.to_string_lossy().to_string()).filter(|f| !existing_paths.contains(f)).collect();

            // {
            //     let mut lock = status.lock().await;
            //     lock.loading_done = 0;
            //     lock.loading_count = folders.len();
            //     lock.custom_message = "Checking folders...".to_owned();
            // }

            trace!("loading from the disk");
            let mut manager = BEATMAP_MANAGER.write().await;
            let len = manager.beatmaps.len();
            
            // this should probably be delegated to the background
            for f in folders.iter() {
                manager.check_folder(f).await;
                // status.lock().await.loading_done += 1;
            }

            let nlen = manager.beatmaps.len();
            debug!("loaded {nlen} beatmaps ({} new)", nlen - len);
        });


    }

}

#[async_trait]
impl AsyncMenu<Game> for LoadingMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
        
    }

    async fn update(&mut self, game:&mut Game) {
        if let LoadingStage::Done = self.status.lock().await.stage {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(crate::tataku::GameState::InMenu(menu));

            // select a map to load bg and intro audio from (TODO! add our own?)
            // let mut manager = BEATMAP_MANAGER.write().await;

            // if let Some(map) = manager.random_beatmap() {
            //     manager.set_current_beatmap(game, &map, false, false);
            // }
            
        }
    }

    async fn draw(&mut self, _args:piston::RenderArgs, list: &mut RenderableCollection) {
        let font = get_font();

        // since this is just loading, we dont care about performance here
        let state = self.status.lock().await;

        let text_color = Color::WHITE;

        let mut text:Text;
        match &state.error {
            Some(error) => {
                text = Text::new(
                    text_color,
                    -100.0,
                    Vector2::ZERO,
                    32,
                    error.clone(),
                    font
                )
            }
            None => match state.stage {
                LoadingStage::None => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::ZERO,
                        32,
                        format!(""),
                        font
                    )
                },
                LoadingStage::Done => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::ZERO,
                        32,
                        format!("Done"),
                        font
                    )
                }
                LoadingStage::Database => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::ZERO,
                        32,
                        format!("Loading Database"),
                        font
                    )
                }
                LoadingStage::Audio => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::ZERO,
                        32,
                        format!("Loading Audio"),
                        font
                    )
                }
                LoadingStage::Beatmaps => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::ZERO,
                        32,
                        format!("{} ({}/{})", 
                            if state.custom_message.is_empty() {"Loading Beatmaps"} else {&state.custom_message},
                            state.loading_done, 
                            state.loading_count
                        ),
                        font
                    )
                }
            },
        }

        text.center_text(&Rectangle::bounds_only(Vector2::ZERO, self.window_size.0));
        list.push(text);
    }
}
impl ControllerInputMenu<Game> for LoadingMenu {}

/// async helper
struct LoadingStatus {
    stage: LoadingStage,
    error: Option<String>,

    loading_count: usize, // items in the list
    loading_done: usize, // items done loading in the list
    custom_message: String
}
impl LoadingStatus {
    pub fn new() -> Self {
        Self {
            error: None,
            loading_count: 0,
            loading_done: 0,
            stage: LoadingStage::None,
            custom_message: String::new()
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LoadingStage {
    None,
    Database,
    Beatmaps,
    Audio,

    Done,
}
