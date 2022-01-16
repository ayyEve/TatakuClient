#![feature(vec_retain_mut)]
use crate::prelude::*;

// include files
mod game;
mod menu;
mod errors;
mod prelude;
mod graphics;
mod beatmaps;
mod gameplay;
mod databases;
pub mod commits;
pub mod skinning;
mod visualization;

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";

// https://cdn.ayyeve.xyz/taiko-rs/
pub const REQUIRED_FILES:&[&str] = &[
    "resources/audio/don.wav",
    "resources/audio/kat.wav",
    "resources/audio/bigdon.wav",
    "resources/audio/bigkat.wav",
    "resources/audio/combobreak.mp3",
    "resources/icon-small.png",
    "resources/icon.png",
    "fonts/main.ttf",
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
    let mut main_benchmark = BenchmarkHelper::new("main");

    // check for missing folders
    check_folder(DOWNLOADS_DIR);
    check_folder(REPLAYS_DIR);
    check_folder(SONGS_DIR);
    check_folder("fonts");
    check_folder("resources");
    check_folder("resources/audio");

    main_benchmark.log("Folder check done, downloading files", true);

    // check for missing files
    for file in REQUIRED_FILES.iter() {
        check_file(file, &format!("https://cdn.ayyeve.xyz/taiko-rs/{}", file)).await;
    }

    // hitsounds
    for sample_set in ["normal", "soft", "drum"] {
        for hitsound in ["hitnormal", "hitwhistle", "hitclap", "hitfinish", "slidertick"] {
            let file = &format!("resources/audio/{}-{}.wav", sample_set, hitsound);
            check_file(file, &format!("https://cdn.ayyeve.xyz/taiko-rs/{}", file)).await;
        }
    }

    
    // check if songs folder is empty
    if std::fs::read_dir(SONGS_DIR).unwrap().count() == 0 {
        // no songs, download some
        for id in FIRST_MAPS {
            check_file(&format!("{}/{}.osz", DOWNLOADS_DIR, id), &format!("https://cdn.ayyeve.xyz/taiko-rs/maps/{}.osz", id)).await;
        }
    }

    // check bass lib
    check_bass().await;

    main_benchmark.log("File check done", true);
    
    let game = Game::new();
    main_benchmark.log("Game creation complete", true);

    drop(main_benchmark);
    game.game_loop();
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
        println!("{:?} not found, attempting to find or download", library_path);
        
        // if linux, check for lib in /usr/lib
        #[cfg(target_os = "linux")]
        if exists(format!("/usr/lib/{}", filename)) {
            match std::fs::copy(filename, &library_path) {
                Ok(_) => return println!("Found in /usr/lib"),
                Err(e) => println!("Found in /usr/lib, but couldnt copy: {}", e)
            }
        }

        // download it from the web
        check_file(&library_path, &format!("https://cdn.ayyeve.xyz/taiko-rs/bass/{}", filename)).await;
    } else {
        println!("error getting current executable dir, assuming things are good...")
    }
}


/// format a number into a locale string ie 1000000 -> 1,000,000
fn format<T:Display>(num:T) -> String {
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
    if dec.is_some() {
        new_str += &format!(".{}", dec.unwrap());
    }

    let mut new_new = String::with_capacity(new_str.len());
    new_new.extend(new_str.chars().rev());
    new_new.trim_start_matches(",").to_owned()
}

