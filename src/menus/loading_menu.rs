use std::fs::read_dir;

use crate::prelude::*;
/// helper for when starting the game. will load beatmaps, settings, etc from storage
/// all while providing the user with its progress (relatively anyways)
pub struct LoadingMenu {
    pub complete: bool,
    status: Arc<Mutex<LoadingStatus>>
}

impl LoadingMenu {
    pub fn new() -> Self {
        Self {
            complete: false,
            status: Arc::new(Mutex::new(LoadingStatus::new()))
        }
    }
    pub fn load(&mut self) {
        let status = self.status.clone();
        
        tokio::spawn(async move {
            let status = status.clone();

            // load database
            Self::load_database(status.clone()).await;

            // preload audio 
            Self::load_audio(status.clone()).await;

            // load beatmaps
            Self::load_beatmaps(status.clone()).await;

            status.lock().stage = LoadingStage::Done;
        });
    }

    // loaders
    async fn load_database(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().stage = LoadingStage::Database;
        // let _ = crate::databases::DATABASE.lock();
    }

    async fn load_audio(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().stage = LoadingStage::Audio;
        // get a value from the hash, will do the lazy_static stuff and populate
        // if let Ok(a) = Audio::play_preloaded("don") {
        //     a.stop();
        // }
    }

    async fn load_beatmaps(status: Arc<Mutex<LoadingStatus>>) {
        status.lock().stage = LoadingStage::Beatmaps;
        // set the count and reset the counter
        status.lock().loading_count = 0;
        status.lock().loading_done = 0;

        let mut dirs_to_check = get_settings!().external_games_folders.clone();
        dirs_to_check.push(SONGS_DIR.to_owned());


        let mut folders = Vec::new();
        for dir in dirs_to_check {
            read_dir(dir)
                .unwrap()
                .for_each(|f| {
                    let f = f.unwrap().path();
                    folders.push(f.to_str().unwrap().to_owned());
                });
        }


        {
            let existing_maps = Database::get_all_beatmaps();
            println!("loading {} from the db", existing_maps.len());
            
            status.lock().loading_count = existing_maps.len();
            // load from db
            let mut lock = BEATMAP_MANAGER.write();
            for meta in existing_maps {
                // verify the map exists
                if !std::path::Path::new(&meta.file_path).exists() {
                    // println!("beatmap exists in db but not in songs folder: {}", meta.file_path);
                    continue
                }

                lock.add_beatmap(&meta);
                status.lock().loading_done += 1;
            }
        }
        
        // look through the songs folder to make sure everything is already added
        BEATMAP_MANAGER.write().initialized = true; // these are new maps

        // get existing dirs
        let mut existing_paths = HashSet::new();
        for i in BEATMAP_MANAGER.read().beatmaps.iter() {
            if let Some(parent) = Path::new(&i.file_path).parent() {
                existing_paths.insert(parent.to_string_lossy().to_string());
            }
        }
        // filter out folders that already exist
        let folders:Vec<String> = folders.into_iter().filter(|f|!existing_paths.contains(f)).collect();

        {
            let mut lock = status.lock();
            lock.loading_done = 0;
            lock.loading_count = folders.len();
            lock.custom_message = "Checking folders...".to_owned();
        }

        folders.par_iter().for_each(|f| {
            BEATMAP_MANAGER.write().check_folder(f);
            status.lock().loading_done += 1;
        });


        println!("loaded {} beatmaps", BEATMAP_MANAGER.read().beatmaps.len())
    }

}

impl Menu<Game> for LoadingMenu {
    fn update(&mut self, game:&mut Game) {
        if let LoadingStage::Done = self.status.lock().stage {
            let menu = game.menus.get("main").unwrap().clone();
            game.queue_state_change(crate::game::GameState::InMenu(menu));

            // select a map to load bg and intro audio from (TODO! add our own?)
            let mut manager = BEATMAP_MANAGER.write();

            if let Some(map) = manager.random_beatmap() {
                manager.set_current_beatmap(game, &map, false, false);
            }
            
        }
    }

    fn draw(&mut self, _args:piston::RenderArgs) -> Vec<Box<dyn Renderable>> {
        let mut list: Vec<Box<dyn Renderable>> = Vec::new();
        let font = get_font();

        // since this is just loading, we dont care about performance here
        let state = self.status.lock();

        let text_color = Color::WHITE;

        let mut text:Text;
        match &state.error {
            Some(error) => {
                text = Text::new(
                    text_color,
                    -100.0,
                    Vector2::zero(),
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
                        Vector2::zero(),
                        32,
                        format!(""),
                        font
                    )
                },
                LoadingStage::Done => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Done"),
                        font
                    )
                }
                LoadingStage::Database => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Loading Database"),
                        font
                    )
                }
                LoadingStage::Audio => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Loading Audio"),
                        font
                    )
                }
                LoadingStage::Beatmaps => {
                    text = Text::new(
                        text_color,
                        -100.0,
                        Vector2::zero(),
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

        text.center_text(Rectangle::bounds_only(Vector2::zero(), Settings::window_size()));
        list.push(Box::new(text));
        list
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
