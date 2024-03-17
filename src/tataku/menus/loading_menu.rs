use crate::prelude::*;

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub statuses: Vec<Arc<RwLock<LoadingStatus>>>,
    // window_size: Arc<WindowSize>,
}

impl LoadingMenu {
    pub async fn new() -> Self {
        Self {
            statuses: Vec::new(),
            
            // window_size: WindowSize::get(),
        }
    }
    pub async fn load(&mut self) {
        macro_rules! add {
            ($fn: ident, $stage: ident) => {{
                let status = Arc::new(RwLock::new(LoadingStatus::new(LoadingStage::$stage)));
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
    async fn load_difficulties(status: Arc<RwLock<LoadingStatus>>) {
        // trace!("loading difficulties");
        // status.lock().await.stage = LoadingStage::Difficulties;
        
        // init diff manager
        init_diffs(Some(status.clone())).await;

        status.write().complete = true;
    }

    async fn load_beatmaps(status: Arc<RwLock<LoadingStatus>>) {
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
            
            status.write().item_count = existing_len;
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
                status.write().items_complete += 1;
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
                let mut lock = status.write();
                lock.items_complete = 0;
                lock.item_count = folders.len();
                lock.custom_message = "Checking folders...".to_owned();
            }

            trace!("loading from the disk");
            let mut manager = BEATMAP_MANAGER.write().await;
            
            // this should probably be delegated to the background
            for f in folders.iter() {
                manager.check_folder(f, true).await;
                status.write().items_complete += 1;
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

        status.write().complete = true;
    }

    async fn init_integrations(status: Arc<RwLock<LoadingStatus>>) {
        let settings = Settings::get();
        status.write().item_count = 2;

        if settings.integrations.lastfm {
            LastFmIntegration::check(&settings).await;
        }
        status.write().items_complete += 1;

        if settings.integrations.discord {
            OnlineManager::init_discord().await;
        }

        status.write().complete = true;
    }

    async fn init_fonts(status: Arc<RwLock<LoadingStatus>>) {
        status.write().item_count = 3;

        #[cfg(feature="graphics")]
        preload_fonts();

        status.write().complete = true;
    }
}

#[async_trait]
impl AsyncMenu for LoadingMenu {
    fn get_name(&self) -> &'static str { "loading_menu" }

    async fn update(&mut self, _values: &mut ValueCollection) -> Vec<TatakuAction> {
        for status in self.statuses.iter() {
            let status = status.read();
            if !status.complete { return Vec::new() }
        }

        // loading complete, move to the main menu
        vec![
            TatakuAction::Beatmap(BeatmapAction::Next),
            TatakuAction::Menu(MenuMenuAction::SetMenu("main_menu".to_owned()))
        ]
        // vec![MenuMenuAction::SetMenu(Box::new(MainMenu::new().await))]
    }

    
    fn view(&self, _values: &mut ValueCollection) -> IcedElement {
        use crate::prelude::iced_elements::*;
        row!(
            Space::new(Fill, Fill),

            col!(
                self.statuses.iter().map(|status| {
                    let status = status.read();

                    let text;
                    let mut color = Color::BLUE;

                    if let Some(error) = &status.error {
                        text = "Error: ".to_owned() + error;
                        color = Color::RED;
                    } else if status.complete {
                        text = "Done".to_owned();
                        color = Color::LIME;
                    } else if !status.custom_message.is_empty() {
                        text = status.custom_message.clone();
                    } else {
                        text = format!("{}/{}", status.items_complete, status.item_count);
                    }
                    
                    row!(
                        Text::new(status.stage.name().to_owned() + ": ").color(Color::WHITE),
                        Text::new(text).color(color).width(Fill);
                        width = Fill
                    )
                }).collect(),
                width = Fill,
                height = Fill,
                spacing = 5.0
            ),

            Space::new(Fill, Fill);
            
            width = Fill,
            height = Fill,
            align_items = Alignment::Center
        )
    }
    
    async fn handle_message(&mut self, _message: Message, _values: &mut ValueCollection) {
        // nothing really to do here
    }
}

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
