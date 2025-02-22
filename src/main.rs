use crate::prelude::*;

#[macro_use]
extern crate log;

// include files
mod engine;
mod tataku;
mod prelude;
pub mod commits;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKIN_FOLDER:&str = "skins";
pub const REPLAY_EXPORTS_DIR:&str = "../replays";
const DOWNLOAD_URL_BASE:&str = "https://cdn.ayyeve.xyz/tataku";

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


// main fn
#[tokio::main]
async fn main() {
    // enter game dir
    if exists("./game") {
        if let Err(e) = std::env::set_current_dir("./game") {
            println!("error changing current dir: {}", e);
        }
    }

    // setup logging
    init_logging();

    // finish setting up
    setup().await;

    // init skin manager
    SkinManager::init().await;


    let mut play_game = true;

    let mut args = std::env::args().map(|s|s.to_string());
    args.next(); // skip the file param

    // let path = std::env::current_exe().unwrap();
    // println!("file hash: {}", get_file_hash(&path).unwrap());

    if let Some(param1) = args.next() {
        match &*param1 {
            "--diff_calc" | "--diffcalc" | "-d" => {
                play_game = false;
                diff_calc_cli(&mut args).await;
            }

            _ => {}
        }
    }

    if play_game {
        start_game().await;

        // game.await.ok().expect("error finishing game?");
        info!("byebye!");
    }

}

async fn start_game() {
    let main_thread = tokio::task::LocalSet::new();

    let (render_queue_sender, render_queue_receiver) = TripleBuffer::default().split();
    let (game_event_sender, game_event_receiver) = tokio::sync::mpsc::channel(30);

    // setup window
    trace!("creating window");
    let mut window = GameWindow::start(render_queue_receiver, game_event_sender);

    // texture load loop
    main_thread.spawn_local(async {
        info!("starting texture load loop");
        texture_load_loop().await;
        warn!("texture loop closed");
    });

    // start game
    let game = tokio::spawn(async move {
        info!("starting wait loop");
        while !TEXTURE_LOAD_QUEUE.initialized() {
            tokio::task::yield_now().await;
            // tokio::time::sleep(Duration::from_millis(100)).await;
        }
        trace!("window ready");

        // start the game
        trace!("creating game");
        let game = Game::new(render_queue_sender, game_event_receiver).await;
        trace!("running game");
        game.game_loop().await;
        warn!("game closed");

        TEXTURE_LOAD_QUEUE.get().unwrap().send(LoadImage::GameClose).ok().unwrap();
    });

    main_thread.spawn_local(async move {
        trace!("window running");
        window.run().await;
        warn!("window closed");
    });

    let _ = tokio::join!(main_thread, game);
}


async fn setup() {
    Settings::load().await;

    // check for missing folders
    check_folder(DOWNLOADS_DIR);
    check_folder(REPLAYS_DIR);
    check_folder(SONGS_DIR);
    check_folder("skins");
    check_folder("resources");
    check_folder("resources/audio");
    check_folder("resources/fonts");

    debug!("Folder check done, downloading files");

    // check for missing files
    for file in REQUIRED_FILES.iter() {
        check_file(file, &download_url(file)).await;
    }

    // hitsounds
    for mode in ["", "taiko-"] {
        for sample_set in ["normal", "soft", "drum"] {
            for hitsound in ["hitnormal", "hitwhistle", "hitclap", "hitfinish", "slidertick"] {
                let file = format!("resources/audio/{mode}{sample_set}-{hitsound}.wav");
                check_file(&file, &download_url(&file)).await;
            }
        }
    }
    
    // check if songs folder is empty
    if std::fs::read_dir(SONGS_DIR).unwrap().count() == 0 {
        // no songs, download some
        for id in FIRST_MAPS {
            check_file(&format!("{}/{}.osz", DOWNLOADS_DIR, id), &download_url(format!("/maps/{}.osz", id))).await;
        }
    }

    // check bass lib
    check_bass().await;

    debug!("File check done");
}


// helper functions

/// check for the bass lib
/// if not found, will be downloaded
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
        if library_path.exists() {return}
        info!("{:?} not found, attempting to find or download", library_path);
        
        // if linux, check for lib in /usr/lib
        #[cfg(target_os = "linux")]
        if exists(format!("/usr/lib/{}", filename)) {
            match std::fs::copy(filename, &library_path) {
                Ok(_) => return info!("Found in /usr/lib"),
                Err(e) => warn!("Found in /usr/lib, but couldnt copy: {}", e)
            }
        }

        // download it from the web
        check_file(&library_path, &download_url(format!("/lib/bass/{}", filename))).await;
    } else {
        warn!("error getting current executable dir, assuming things are good...")
    }
}


fn init_logging() {
    // start log handler
    tataku_logging::init_with_level("logs/", log::Level::Info).unwrap();

    // clean up any old log files
    let Ok(files) = std::fs::read_dir("logs/") else { return };
    let today = chrono::Utc::now().date_naive();
    let Some(remove_date) = today.checked_sub_days(chrono::Days::new(5)) else { return warn!("Unable to determine log remove date")};

    for file in files.filter_map(|f|f.ok()) {
        let filename = file.file_name();
        let Some(date) = filename.to_str().and_then(|s|s.split("--").next()) else { continue };
        let Ok(date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") else { continue };

        if date <= remove_date {
            if let Err(e) = std::fs::remove_file(file.path()) {
                error!("Error removing log file: {e}");
            }
        }
    }
}