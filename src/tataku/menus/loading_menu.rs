use crate::prelude::*;

const STATUS_HEIGHT:f32 = 50.0;
const STATUS_MARGIN:f32 = 5.0;

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub statuses: Vec<Arc<AsyncRwLock<LoadingStatus>>>,
    window_size: Arc<WindowSize>,
}

impl LoadingMenu {
    pub async fn new() -> Self {
        Self {
            statuses: Vec::new(),
            
            window_size: WindowSize::get(),
        }
    }
    pub async fn load(&mut self) {
        macro_rules! add {
            ($fn: ident, $stage: ident) => {{
                let status = Arc::new(AsyncRwLock::new(LoadingStatus::new(LoadingStage::$stage)));
                self.statuses.push(status.clone());
                tokio::spawn(Self::$fn(status));
            }}
        }

        // load difficulties
        add!(load_difficulties, Difficulties);

        // load beatmaps
        add!(load_beatmaps, Beatmaps);

        // init integrations
        add!(init_integrations, Integrations);

        // init fonts
        add!(init_fonts, Fonts);
    }

    // loaders
    async fn load_difficulties(status: Arc<AsyncRwLock<LoadingStatus>>) {
        // trace!("loading difficulties");
        // status.lock().await.stage = LoadingStage::Difficulties;
        
        // init diff manager
        init_diffs(Some(status.clone())).await;

        status.write().await.complete = true;
    }

    async fn load_beatmaps(status: Arc<AsyncRwLock<LoadingStatus>>) {
        // trace!("loading beatmaps");
        // status.lock().await.stage = LoadingStage::Beatmaps;
        // set the count and reset the counter
        // status.lock().await.loading_count = 0;
        // status.lock().await.loading_done = 0;


        let ignored = Database::get_all_ignored().await;
        let existing_len;
        trace!("got ignored {}", ignored.len());

        {
            let existing_maps = Database::get_all_beatmaps().await;
            existing_len = existing_maps.len();
            trace!("loading {existing_len} from the db");
            
            status.write().await.item_count = existing_len;
            // load from db
            let mut lock = BEATMAP_MANAGER.write().await;
            lock.ignore_beatmaps = ignored.into_iter().collect();

            for meta in existing_maps {
                // verify the map exists
                if !std::path::Path::new(&*meta.file_path).exists() {
                    trace!("beatmap exists in db but not in fs: {}", meta.file_path);
                    continue
                }

                lock.add_beatmap(&meta);
                status.write().await.items_complete += 1;
            }
            trace!("done beatmap manager init");
            lock.initialized = true;
        }
        
        // look through the songs folder to make sure everything is already added
        if existing_len == 0 {
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

            {
                let mut lock = status.write().await;
                lock.items_complete = 0;
                lock.item_count = folders.len();
                lock.custom_message = "Checking folders...".to_owned();
            }

            trace!("loading from the disk");
            let mut manager = BEATMAP_MANAGER.write().await;
            
            // this should probably be delegated to the background
            for f in folders.iter() {
                manager.check_folder(f, true).await;
                status.write().await.items_complete += 1;
            }

            let nlen = manager.beatmaps.len();
            debug!("loaded {nlen} beatmaps ({} new)", nlen - existing_len);
        }

        
        // {
        //     let beatmaps = BEATMAP_MANAGER.read().await.beatmaps.clone();
        //     let timer = std::time::Instant::now();
        //     for b in beatmaps.iter() {
        //         if b.beatmap_type == BeatmapType::Osu {
        //             let _ = OsuBeatmap::load(b.file_path.clone());
        //         }
        //     }
        //     let full_elapsed = timer.elapsed().as_secs_f32() * 1000.0;
        //     let timer = std::time::Instant::now();

        //     for b in beatmaps.iter() {
        //         if b.beatmap_type == BeatmapType::Osu {
        //             let _ = OsuBeatmap::load_metadata(b.file_path.clone());
        //         }
        //     }
        //     let meta_elapsed = timer.elapsed().as_secs_f32() * 1000.0;
            
        //     println!("full took {full_elapsed:.4}");
        //     println!("meta took {meta_elapsed:.4}")
        // }

        status.write().await.complete = true;
    }

    async fn init_integrations(status: Arc<AsyncRwLock<LoadingStatus>>) {
        let settings = Settings::get();
        status.write().await.item_count = 2;

        if settings.integrations.lastfm {
            LastFmIntegration::check(&settings).await;
        }
        status.write().await.items_complete += 1;

        if settings.integrations.discord {
            OnlineManager::init_discord().await;
        }

        status.write().await.complete = true;
    }

    async fn init_fonts(status: Arc<AsyncRwLock<LoadingStatus>>) {
        status.write().await.item_count = 3;

        #[cfg(feature="graphics")]
        preload_fonts();

        status.write().await.complete = true;
    }
}

#[async_trait]
impl AsyncMenu<Game> for LoadingMenu {
    async fn window_size_changed(&mut self, window_size: Arc<WindowSize>) {
        self.window_size = window_size;
    }

    async fn update(&mut self, game:&mut Game) {
        for status in self.statuses.iter() {
            let status = status.read().await;
            if !status.complete { return }
        }

        // let menu = game.menus.get("main").unwrap().clone();

        // select a map to load bg and intro audio from (TODO! add our own?)
        // let mut manager = BEATMAP_MANAGER.write().await;

        // if let Some(map) = manager.random_beatmap() {
        //     manager.set_current_beatmap(game, &map, false, false);
        // }
        game.queue_state_change(GameState::InMenu(Box::new(MainMenu::new().await)));
    }

    async fn draw(&mut self, list: &mut RenderableCollection) {
        let count = self.statuses.len() as f32;
        let mut y = (self.window_size.y - (STATUS_HEIGHT + STATUS_MARGIN) * count) / 2.0;

        for status in self.statuses.iter() {
            let bounds = Bounds::new(Vector2::with_y(y), Vector2::new(self.window_size.x, STATUS_HEIGHT));
            y += STATUS_HEIGHT + STATUS_MARGIN;
            
            
            let status = status.read().await;

            let mut text = status.stage.name().to_owned() + ": ";
            let stage_text_len = text.len();
            let mut status_text_color = Color::BLUE;

            if let Some(error) = &status.error {
                text += &("Error: ".to_owned() + error);
                status_text_color = Color::RED;
            } else if status.complete {
                text += "Done";
                status_text_color = Color::LIME;
            } else if !status.custom_message.is_empty() {
                text += &status.custom_message;
            } else {
                text += &format!("{}/{}", status.items_complete, status.item_count);
            }

            let mut text = Text::new(Vector2::ZERO, STATUS_HEIGHT * 0.8, text, Color::WHITE, Font::Main);
            text.set_text_colors(
                (0..stage_text_len).into_iter().map(|_|Color::WHITE).chain(
                (stage_text_len..text.text.len()).into_iter().map(|_|status_text_color)
                ).collect()
            );
            text.center_text(&bounds);
            list.push(text);
        }
    }
}
impl ControllerInputMenu<Game> for LoadingMenu {}

/// async helper
pub struct LoadingStatus {
    stage: LoadingStage,
    pub error: Option<String>,

    pub item_count: usize, // items in the list
    pub items_complete: usize, // items done loading in the list
    pub custom_message: String,

    pub complete: bool,
}
impl LoadingStatus {
    pub fn new(stage: LoadingStage) -> Self {
        Self {
            error: None,
            item_count: 0,
            items_complete: 0,
            stage,
            custom_message: String::new(),

            complete: false
        }
    }

}

#[derive(Clone, Copy, Debug)]
pub enum LoadingStage {
    Difficulties,
    Beatmaps,
    Integrations,
    Fonts,
}
impl LoadingStage {
    fn name(&self) -> &'static str {
        match self {
            Self::Difficulties => "Loading difficulties",
            Self::Beatmaps => "Loading beatmaps",
            Self::Integrations => "Initializing integrations",
            Self::Fonts => "Initializing fonts",
        }
    }
}