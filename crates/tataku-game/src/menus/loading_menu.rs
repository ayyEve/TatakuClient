use crate::prelude::*;
use crate::prelude::ui::*;

/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    actions: ActionQueue,
    pub statuses: Vec<Arc<RwLock<LoadingStatus>>>,
    // window_size: Arc<WindowSize>,

    node: Box<dyn Widget>,
    node_id: NodeId,
}

impl LoadingMenu {
    pub async fn new() -> Self {
        Self {
            actions: ActionQueue::new(),
            statuses: Vec::new(),
            node: Box::new(EmptyWidget::new()),
            node_id: EMPTY_NODE
            // window_size: WindowSize::get(),
        }
    }
    pub async fn load(&mut self, settings: &Settings) {
        macro_rules! add {
            ($fn: ident, $stage: expr) => {{
                let status = Arc::new(RwLock::new(LoadingStatus::new($stage)));
                self.statuses.push(status.clone());
                tokio::spawn(Self::$fn(status));
            }}
        }

        {
            let status = Arc::new(RwLock::new(LoadingStatus::new("Loading beatmaps")));
            self.actions.push(TaskAction::AddTask(Box::new(LoadBeatmapsTask::new(status.clone()))));
            self.statuses.push(status);
        }

        // init integrations
        {
            let settings = settings.clone();
            let status = Arc::new(RwLock::new(LoadingStatus::new("Initializing integrations")));
            tokio::spawn(Self::init_integrations(status.clone(), settings));
            self.statuses.push(status);
        }

        // init fonts
        add!(init_fonts, "Initializing fonts");
    }

    fn build_view(&self) -> Box<dyn Widget> {

        let elements = self.statuses.iter()
            .map(|status| {
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
                TextWidget::new(status.name.to_owned() + ": ").text_color(Color::WHITE).boxed(),
                TextWidget::new(text).text_color(color).width(FILL).boxed()
                ;
                width = FILL
            )
        }).collect::<Vec<_>>();

        row!(
            Space::new(FILL, FILL).boxed(),

            col!(
                elements,
                width = FILL,
                height = FILL
                // spacing = 5.0
            ),

            Space::new(FILL, FILL).boxed();
            
            width = FILL,
            height = FILL,
            vertical_align = AlignContent::Center
        )
    }

    // loaders
    // async fn load_difficulties(status: Arc<RwLock<LoadingStatus>>) {
    //     // trace!("loading difficulties");
    //     // status.lock().await.stage = LoadingStage::Difficulties;
        
    //     // init diff manager
    //     init_diffs(Some(status.clone())).await;

    //     status.write().complete = true;
    // }

    /*
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
    */

    async fn init_integrations(status: Arc<RwLock<LoadingStatus>>, settings: Settings) {
        status.write().item_count = 2;

        if settings.integrations.lastfm {
            LastFmIntegration::check(&settings).await;
        }
        status.write().items_complete += 1;

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
impl Widget for LoadingMenu {
    fn name(&self) -> Cow<'static, str> { Cow::Borrowed("loading_menu") }
    fn node_id(&self) -> NodeId { self.node_id }

    fn layout(&mut self, shell: &mut LayoutShell<'_>) -> TaffyResult<NodeId> {
        self.node = self.build_view();

        let child = self.node.layout(shell)?;
        self.node_id = shell.tree.new_with_children(
            Style::default(), 
            &[child]
        )?;

        Ok(self.node_id)
    }

    fn update(
        &mut self, 
        _shell: &mut UpdateShell<'_>, 
        actions: &mut ActionQueue
    ) {
        actions.extend(self.actions.take());

        for status in self.statuses.iter() {
            let status = status.read();
            if !status.complete { return }
        }

        // loading complete, move to the main menu
        actions.push(BeatmapAction::Next);
        actions.push(MenuAction::set_menu("main_menu"));
    }

    fn draw(
        &self,
        shell: &mut DrawShell<'_>,
    ) {
        self.node.draw(shell);
    }
}

