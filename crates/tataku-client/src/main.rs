use tataku_game::prelude::*;

const DOWNLOAD_URL_BASE:&str = "https://cdn.ayyeve.dev/tataku";

mod game;


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
    let _guards = init_logging();
    startup();
    start_game();
}

fn start_game() {
    let (
        game_event_sender, 
        game_event_receiver
    ) = tokio::sync::mpsc::channel(30);
    let window_load_barrier = Arc::new(std::sync::Barrier::new(2));
    let window_side_barrier = window_load_barrier.clone();

    let e = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    let proxy = e.create_proxy();

    // start game
    let game = std::thread::spawn(move || {
        // wait for the window side to be ready
        // window_load_barrier.wait().await;
        window_load_barrier.wait();
        trace!("window ready");

        game::run_game(
            game_event_receiver,
            proxy,
        );
    });

    static WINDOW: tokio::sync::OnceCell<winit::window::Window> = tokio::sync::OnceCell::const_new();

    // setup window
    info!("creating window");
    let settings = Settings::load();
    let game_window = GameWindow::new(
        game_event_sender,
        &WINDOW,
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
    );


    trace!("window running");
    game_window.run(e);

    // // wait for game to finish
    // runtime.block_on(game).unwrap();
    game.join().unwrap();

    info!("Byebye!");
}


fn startup() {
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
    setup();
}

fn setup() {
    trace!("Client setup");
    Settings::load();

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
        Io::check_file_sync(file, &download_url(file));
    }

    // hitsounds
    for mode in ["", "taiko-"] {
        for sample_set in ["normal", "soft", "drum"] {
            for hitsound in ["hitnormal", "hitwhistle", "hitclap", "hitfinish", "slidertick"] {
                let file = format!("resources/audio/{mode}{sample_set}-{hitsound}.wav");
                Io::check_file_sync(&file, &download_url(&file));
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
