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
        let _ = crate::databases::DATABASE.lock();
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

        let mut folders = Vec::new();
        read_dir(SONGS_DIR)
            .unwrap()
            .for_each(|f| {
                let f = f.unwrap().path();
                folders.push(f.to_str().unwrap().to_owned());
            });


        {
            let mut db = crate::databases::DATABASE.lock();
            let t = db.transaction().unwrap();
            let mut s = t.prepare("SELECT * FROM beatmaps").unwrap();

            let rows = s.query_map([], |r| {
                let meta = BeatmapMeta {
                    file_path: r.get("beatmap_path")?,
                    beatmap_hash: r.get("beatmap_hash")?,
                    beatmap_version: r.get("beatmap_version")?,
                    mode: r.get("playmode")?,
                    artist: r.get("artist")?,
                    title: r.get("title")?,
                    artist_unicode: r.get("artist_unicode")?,
                    title_unicode: r.get("title_unicode")?,
                    creator: r.get("creator")?,
                    version: r.get("version")?,
                    audio_filename: r.get("audio_filename")?,
                    image_filename: r.get("image_filename")?,
                    audio_preview: r.get("audio_preview")?,
                    hp: r.get("hp")?,
                    od: r.get("od")?,
                    ar: r.get("ar")?,
                    cs: r.get("cs")?,
                    slider_multiplier: r.get("slider_multiplier")?,
                    slider_tick_rate: r.get("slider_tick_rate")?,
                    stack_leniency: r.get("stack_leniency").unwrap_or(0.0),
        
                    duration: r.get("duration")?,
                    bpm_min: r.get("bpm_min").unwrap_or(0.0),
                    bpm_max: r.get("bpm_max").unwrap_or(0.0),

                    // TODO: store this
                    diff: -1.0
                };

                Ok(meta)
            })
                .unwrap()
                .filter_map(|m|m.ok())
                .collect::<Vec<BeatmapMeta>>();

            
            status.lock().loading_count = rows.len();
            // load from db
            let mut lock = BEATMAP_MANAGER.write();
            for meta in rows {
                // verify the map exists
                if !std::path::Path::new(&meta.file_path).exists() {
                    // println!("beatmap exists in db but not in songs folder: {}", meta.file_path);
                    continue
                }

                status.lock().loading_done += 1;
                lock.add_beatmap(&meta);
            }
        }
        
        // look through the songs folder to make sure everything is already added
        BEATMAP_MANAGER.write().initialized = true; // these are new maps
        status.lock().loading_count += folders.len();

        for f in folders {
            BEATMAP_MANAGER.write().check_folder(f);
            status.lock().loading_done += 1;
        }

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

        let mut text:Text;
        match &state.error {
            Some(error) => {
                text = Text::new(
                    Color::BLACK,
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
                        Color::BLACK,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!(""),
                        font
                    )
                },
                LoadingStage::Done => {
                    text = Text::new(
                        Color::BLACK,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Done"),
                        font
                    )
                }
                LoadingStage::Database => {
                    text = Text::new(
                        Color::BLACK,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Loading Database"),
                        font
                    )
                }
                LoadingStage::Audio => {
                    text = Text::new(
                        Color::BLACK,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Loading Audio"),
                        font
                    )
                }
                LoadingStage::Beatmaps => {
                    text = Text::new(
                        Color::BLACK,
                        -100.0,
                        Vector2::zero(),
                        32,
                        format!("Loading Beatmaps ({}/{})", state.loading_done, state.loading_count),
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
    loading_done: usize // items done loading in the list
}
impl LoadingStatus {
    pub fn new() -> Self {
        Self {
            error: None,
            loading_count: 0,
            loading_done: 0,
            stage: LoadingStage::None,
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
