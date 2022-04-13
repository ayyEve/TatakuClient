// #![feature(vec_retain_mut)]
use crate::prelude::*;

use tokio::runtime::*;

#[macro_use]
extern crate log;

// include files
mod game;
mod menus;
mod errors;
mod prelude;
mod graphics;
mod beatmaps;
mod gameplay;
mod databases;
pub mod commits;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";
pub const SKIN_FOLDER:&str = "skins";

const DOWNLOAD_URL_BASE:&str = "https://cdn.ayyeve.xyz/tataku";
#[inline]
fn download_url<T:AsRef<str>>(file:T) -> String {
    format!("{}/{}", DOWNLOAD_URL_BASE, file.as_ref())
}

// https://cdn.ayyeve.xyz/taiko-rs/
pub const REQUIRED_FILES:&[&str] = &[

    // default audio
    "resources/audio/don.wav",
    "resources/audio/kat.wav",
    "resources/audio/bigdon.wav",
    "resources/audio/bigkat.wav",
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
fn main() {
    tataku_logging::init("logs/").unwrap();

    if exists("./game") {
        if let Err(e) = std::env::set_current_dir("./game") {
            error!("error changing current dir: {}", e);
        }
    }

    let main_thread = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("error creating main thread");
    

    let multi_thread = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("error creating multi thread");
    
    let ready = Arc::new(AtomicBool::new(false));
    let ready2 = ready.clone();

    // finish setting up
    main_thread.block_on(async move {
        setup().await;
    });

    // init skin manager
    SkinManager::init();

    let (render_queue_sender, render_queue_receiver) = TripleBuffer::default().split();
    let (game_event_sender, game_event_receiver) = MultiBomb::new();

    // setup window
    trace!("creating window");
    let mut window = GameWindow::start(render_queue_receiver, game_event_sender);

    ready2.store(true, SeqCst);

    // enter async runtime
    let _ = multi_thread.enter();
    multi_thread.spawn(async move {
        while !ready.load(SeqCst) {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        trace!("window ready");

        // start the game
        trace!("creating game");
        let game = Game::new(render_queue_sender, game_event_receiver).await;
        trace!("running game");
        game.game_loop().await;
        trace!("game closed");
    });

    trace!("window running");
    main_thread.block_on(async move {
        window.run().await;
    });
    trace!("window closed");

    main_thread.shutdown_timeout(Duration::from_millis(500));
    multi_thread.shutdown_timeout(Duration::from_millis(500));

    info!("byebye!");
    // loop {}
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

    info!("Folder check done, downloading files");

    // check for missing files
    for file in REQUIRED_FILES.iter() {
        check_file(file, &download_url(file)).await;
    }

    // hitsounds
    for sample_set in ["normal", "soft", "drum"] {
        for hitsound in ["hitnormal", "hitwhistle", "hitclap", "hitfinish", "slidertick"] {
            let file = &format!("resources/audio/{}-{}.wav", sample_set, hitsound);
            check_file(file, &download_url(file)).await;
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

    info!("File check done");
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


/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_number<T:Display>(num:T) -> String {
    let str = format!("{}", num);
    let mut split = str.split(".");
    let num = split.next().unwrap();
    let dec = split.next();

    // split num into 3s
    let mut new_str = String::new();
    let offset = num.len() % 3;

    num.char_indices().rev().for_each(|(pos, char)| {
        new_str.push(char);
        if pos % 3 == offset {
            new_str.push(',');
        }
    });

    let mut new_new = String::with_capacity(new_str.len());
    new_new.extend(new_str.chars().rev());
    if let Some(dec) = dec {
        new_new += &format!(".{}", dec);
    }
    new_new.trim_start_matches(",").to_owned()
}

/// format a number into a locale string ie 1000000 -> 1,000,000
pub fn format_float<T:Display>(num:T, precis: usize) -> String {
    let str = format!("{}", num);
    let mut split = str.split(".");
    let num = split.next().unwrap();
    let dec = split.next();

    // split num into 3s
    let mut new_str = String::new();
    let offset = num.len() % 3;

    num.char_indices().rev().for_each(|(pos, char)| {
        new_str.push(char);
        if pos % 3 == offset {
            new_str.push(',');
        }
    });

    let mut new_new = String::with_capacity(new_str.len());
    new_new.extend(new_str.chars().rev());
    if let Some(dec) = dec {
        let dec = if dec.len() < precis {
            format!("{}{}", dec, "0".repeat(precis - dec.len()))
        } else {
            dec.split_at(precis.min(dec.len())).0.to_owned()
        };
        new_new += &format!(".{}", dec);
    } else if precis > 0 {
        new_new += & format!(".{}", "0".repeat(precis))
    }
    new_new.trim_start_matches(",").to_owned()
}

// because rust broke the feature somehow
pub trait RetainMut<T> {
    fn retain_mut<F>(&mut self, f: F) where F:FnMut(&mut T) -> bool;
}
impl<T> RetainMut<T> for Vec<T> {
    fn retain_mut<F>(&mut self, mut f: F) where F:FnMut(&mut T) -> bool {
        *self = std::mem::take(self)
            .into_iter()
            .filter_map(|mut t| if f(&mut t) {Some(t)} else {None})
            .collect()
    }
}