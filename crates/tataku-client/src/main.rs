#![cfg_attr(not(feature="graphics"), allow(unused))]
use tataku_game::prelude::*;

use tracing::*;


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

const FIRST_MAPS: &[u32] = &[
    75, // disco prince (std)
    905576, // triumph and regret (mania)
    1605148, // mayday (std)
    727903, // galaxy collapse (taiko)
];

#[cfg(not(feature="gameplay"))]
fn main() { panic!("should not be running main when not built for gameplay") }

#[cfg(feature="gameplay")]
fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _guards = init_logging();

    // initialize the game
    runtime.block_on(startup());

    #[cfg(feature="gameplay")]
    start_game(&runtime);
}

// // actuial main fn
// fn game_main(runtime: &tokio::runtime::Runtime) {
//     let mut play_game = true;

//     let mut args = std::env::args().map(|s|s.to_string());
//     args.next(); // skip the file param

//     // let path = std::env::current_exe().unwrap();
//     // println!("file hash: {}", get_file_hash(&path).unwrap());

//     // TODO: reimplement this? or do we want to bother
//     // it might be nicer to have a server-side api for it
//     /*
//     if let Some(param1) = args.next() {
//         match &*param1 {
//             "--diff_calc" | "--diffcalc" | "-d" => {
//                 play_game = false;
//                 diff_calc_cli(&mut args).await;
//             }

//             _ => {}
//         }
//     }
//     */

//     if play_game {
//         start_game(runtime);
//         info!("byebye!");
//     }

// }


#[cfg(feature="gameplay")]
fn start_game<'window>(
    runtime: &tokio::runtime::Runtime,
) {
    // let main_thread = tokio::task::LocalSet::new();
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
        #[cfg(feature="graphics")] {
            window_load_barrier.wait().await;
            trace!("window ready");
        }


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
                #[cfg(feature="bass_audio")] Box::new(tataku_bass::BassAudioInit),
            ],
            gamemodes,
        ).await;
        trace!("running game");
        game.game_loop().await;
        warn!("game closed");

        // this shouldnt be necessary but its here commented out just in case
        // GameWindow::send_event(Game2WindowEvent::CloseGame);
    });


    static WINDOW: tokio::sync::OnceCell<winit::window::Window> = tokio::sync::OnceCell::const_new();

    // setup window
    #[cfg(feature="graphics")]
    let game_window = window_runtime.block_on(async {
        info!("creating window");
        let settings = Settings::load(&mut ActionQueue::new()).await;

        GameWindow::new(
            game_event_sender,
            &WINDOW,
            window_runtime.clone(),
            window_side_barrier,
            &settings,
            vec![
                Box::new(tataku_wgpu::WgpuInit)
            ]
        ).await
    });


    trace!("window running");
    #[cfg(feature="graphics")]
    game_window.run(e);

    // wait for game to finish
    runtime.block_on(game).unwrap();

    info!("Byebye!");
}


async fn startup() {
    #[cfg(not(feature="graphics"))] panic!("The client is **NOT** designed to be run without the graphics feature, it will break, and therefor i will not let you do it.");

    // enter game dir
    const GAME_DIR:&str = "./game";

    if !Io::exists(GAME_DIR) {
        if let Err(e) = std::fs::create_dir_all(GAME_DIR) {
            println!("Error creating game dir: {e}");
        }
    }
    if let Err(e) = std::env::set_current_dir(GAME_DIR) {
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

    // check if songs folder is empty
    if std::fs::read_dir(SONGS_DIR).unwrap().count() == 0 {
        // no songs, download some
        for id in FIRST_MAPS {
            Io::check_file(&format!("{}/{}.osz", DOWNLOADS_DIR, id), &download_url(format!("/maps/{}.osz", id))).await;
        }
    }

    // check bass lib
    #[cfg(feature="bass_audio")]
    check_bass().await;

    debug!("File check done");
}




// helper functions

/// check for the bass lib
/// if not found, will be downloaded
#[cfg(feature="bass_audio")]
async fn check_bass() {
    #[cfg(target_os = "windows")]
    let filename = "bass.dll";

    #[cfg(target_os = "linux")]
    let filename = "libbass.so";

    #[cfg(target_os = "macos")]
    let filename = "libbass.dylib";

    if let Ok(mut library_path) = std::env::current_exe() {
        library_path.pop();
        library_path.push(filename);

        // check if already exists
        if library_path.exists() { return }
        info!("{:?} not found, attempting to find or download", library_path);

        // if linux, check for lib in /usr/lib
        #[cfg(target_os = "linux")]
        if Io::exists(format!("/usr/lib/{}", filename)) {
            match std::fs::copy(filename, &library_path) {
                Ok(_) => return info!("Found in /usr/lib"),
                Err(e) => warn!("Found in /usr/lib, but couldnt copy: {}", e)
            }
        }

        // download it from the web
        Io::check_file(&library_path, &download_url(format!("/lib/bass/{}", filename))).await;
    } else {
        warn!("error getting current executable dir, assuming things are good...")
    }
}

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

    const LOG_DIR: &'static str = "game/logs/";
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
        "tataku-client",
        "tataku-game",
        "tataku-engine",
        "tataku-wgpu",
        "tataku-bass",
        "tataku-common",
        "gamemode-osu",
        "gamemode-taiko",
        "gamemode-mania",
        "gamemode-utyping"
    ];

    tracing_subscriber::registry()
        .with(Layer::new()
            .pretty()
            .with_ansi(false)
            .with_writer(trace_file)
            .with_filter(Targets::new()
                .with_default(LevelFilter::INFO)
                .with_targets(tataku_crates.iter().map(|&c| (c, LevelFilter::TRACE)))
            )
        )
        .with(Layer::new()
            .pretty()
            .with_ansi(true)
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

