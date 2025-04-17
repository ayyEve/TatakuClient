use tataku_game::prelude::*;

const DOWNLOAD_URL_BASE:&str = "https://cdn.ayyeve.dev/tataku";

#[inline]
fn download_url<T:AsRef<str>>(file:T) -> String {
    format!("{}/{}", DOWNLOAD_URL_BASE, file.as_ref())
}

pub const REQUIRED_FILES:&[&str] = &[
    // default audio
    "resources/audio/combobreak.mp3",

    // icons
    "resources/icon-small.png",
    "resources/icon.png",

    // fonts
    "resources/fonts/main.ttf",
    "resources/fonts/main_fallback.ttf",
    "resources/fonts/font_awesome_6_regular.otf",
];


fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _guards = init_logging();

    // initialize the game
    runtime.block_on(startup());

    start_game(&runtime);
}

fn start_game(
    runtime: &tokio::runtime::Runtime,
) {
    let window_runtime = Rc::new(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap());

    let (game_event_sender, game_event_receiver) = tokio::sync::mpsc::channel(30);
    let window_load_barrier = Arc::new(tokio::sync::Barrier::new(2));
    let window_side_barrier = window_load_barrier.clone();

    let e = winit::event_loop::EventLoop::with_user_event().build().unwrap();
    let proxy = e.create_proxy();

    // start game
    let game = runtime.spawn(async move {
        // wait for the window side to be ready
        window_load_barrier.wait().await;
        trace!("window ready");


        let gamemodes;
        #[cfg(feature="dynamic_gamemodes")] {
            gamemodes = vec![
                GamemodeLibrary::load_gamemode("/home/ayyeve/Desktop/projects/tataku/tataku-client/target/release/gamemode_taiko").unwrap(),
            ];
        }

        #[cfg(not(feature="dynamic_gamemodes"))] {
            gamemodes = vec![
                gamemode_osu::GAME_INFO,
                gamemode_taiko::GAME_INFO,
                gamemode_mania::GAME_INFO,
                gamemode_utyping::GAME_INFO,
            ]
        }

        // start the game
        trace!("creating game");
        let game = Game::new(
            game_event_receiver,
            proxy,
            vec![
                #[cfg(feature="kira_audio")] tataku_kira::KiraAudioInit, 
                #[cfg(feature="bass_audio")] tataku_bass::BassAudioInit,
            ],
            gamemodes,
        ).await;
        
        trace!("running game");
        game.game_loop().await;
        warn!("game closed");
    });


    static WINDOW: tokio::sync::OnceCell<winit::window::Window> = tokio::sync::OnceCell::const_new();

    // setup window
    let runtime2 = window_runtime.clone();
    let game_window = window_runtime.block_on(async move {
        info!("creating window");
        let settings = Settings::load(&mut ActionQueue::new()).await;

        GameWindow::new(
            game_event_sender,
            &WINDOW,
            runtime2,
            window_side_barrier,
            &settings,
            WindowInitializers {
                integrations: vec![
                    #[cfg(feature="discord")] integration_discord::Discord::builder(),
                    #[cfg(feature="lastfm")] integration_lastfm::LastFm::builder(),
                    #[cfg(feature="media_controls")] integration_media_controls::MediaControlsIntegration::builder(),
                ],
                graphics_init: vec![
                    Box::new(tataku_wgpu::WgpuInit)
                ],
            }
        ).await
    });


    trace!("window running");
    game_window.run(e);

    // wait for game to finish
    runtime.block_on(game).unwrap();

    info!("Byebye!");
}


async fn startup() {
    // enter game dir
    const GAME_DIR:&str = "./game";

    let game_dir = std::env::var("GAME_DIR")
        .unwrap_or(GAME_DIR.to_owned());

    if !Io::exists(&game_dir) {
        if let Err(e) = std::fs::create_dir_all(&game_dir) {
            println!("Error creating game dir: {e}");
        }
    }
    if let Err(e) = std::env::set_current_dir(&game_dir) {
        println!("Error changing current dir: {e}");
    }

    // finish setting up
    setup().await;
}

async fn setup() {
    trace!("Client setup");
    let mut queue = ActionQueue::default();
    Settings::load(&mut queue).await;

    if let Some(queue) = Some(queue.take()).filter(|v| !Vec::is_empty(v)) {
        panic!("error?? {queue:?}")
    }

    // check for missing folders
    debug!("checking folders");
    Io::check_folder(DOWNLOADS_DIR).unwrap();
    Io::check_folder(REPLAYS_DIR).unwrap();
    Io::check_folder(SONGS_DIR).unwrap();
    Io::check_folder("skins").unwrap();
    Io::check_folder("resources").unwrap();
    Io::check_folder("resources/audio").unwrap();
    Io::check_folder("resources/fonts").unwrap();

    debug!("Folder check done, downloading files");

    // check for missing files
    for file in REQUIRED_FILES.iter() {
        Io::check_file(file, &download_url(file)).await;
    }

    // hitsounds
    for mode in ["", "taiko-"] {
        for sample_set in ["normal", "soft", "drum"] {
            for hitsound in ["hitnormal", "hitwhistle", "hitclap", "hitfinish", "slidertick"] {
                let file = format!("resources/audio/{mode}{sample_set}-{hitsound}.wav");
                Io::check_file(&file, &download_url(&file)).await;
            }
        }
    }

    debug!("File check done");
}

// helper functions
#[must_use]
struct LogGuard {
    _guards: [tracing_appender::non_blocking::WorkerGuard; 2],
}

fn init_logging() -> LogGuard {
    use tracing_subscriber::{
        fmt::Layer, layer::SubscriberExt, prelude::*,
        filter::{ LevelFilter, Targets },
    };

    use tracing_appender::{
        non_blocking,
        rolling::{ RollingFileAppender, Rotation },
    };

    const LOG_DIR: &str = "game/logs/";
    const MAX_DAYS: usize = 2;

    let trace_file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("trace-log")
        .filename_suffix(".ttk_log")
        .max_log_files(MAX_DAYS)
        .build(LOG_DIR)
        .unwrap();

    let (trace_file, trace_guard) = non_blocking(trace_file);

    let (stdout, stdout_guard) = non_blocking(std::io::stdout());

    let tataku_crates = [
        "tataku_client",
        "tataku_client_common",
        "tataku_game",
        "tataku_engine",
        "tataku_wgpu",
        "tataku_bass",
        "tataku_common",
        "tataku_input",
        "tataku_interface",

        "gamemode_osu",
        "gamemode_taiko",
        "gamemode_mania",
        "gamemode_utyping",

        "integration_discord",
        "integration_lastfm",
    ];

    let date_format = tracing_subscriber::fmt::time::ChronoLocal::new("%H:%M:%S %z".to_string());

    tracing_subscriber::registry()
        .with(Layer::new()
            .pretty()
            .with_ansi(false)
            .with_timer(date_format.clone())
            .with_writer(trace_file)
            .with_filter(Targets::new()
                .with_default(LevelFilter::INFO)
                .with_targets(tataku_crates.iter().map(|&c| (c, LevelFilter::TRACE)))
            )
        )
        .with(Layer::new()
            .pretty()
            .with_ansi(true)
            .with_timer(date_format.clone())
            .with_writer(stdout)
            .with_filter(Targets::new()
                .with_default(LevelFilter::INFO)
                .with_targets(tataku_crates.iter().map(|&c| (c, LevelFilter::DEBUG)))
            )
        )
        .init();

    LogGuard {
        _guards: [ trace_guard, stdout_guard ]
    }
}
