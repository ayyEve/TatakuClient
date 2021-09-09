// native imports
use std::fmt::Display;
use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufReader, Lines};

use game::Settings;
// local imports
use game::{Game, helpers::BenchmarkHelper};
pub use ayyeve_piston_ui::render;
pub use ayyeve_piston_ui::render::Vector2;
pub use game::helpers;

// include files
mod game;
mod menu;
mod errors;
mod gameplay;
mod databases;
mod visualization;

// re-exports to make imports nicer
mod sync {
    pub use std::sync::{Arc, Weak};
    pub use parking_lot::Mutex;
}

// constants
// const window_size():Vector2 = Vector2::new(1000.0, 600.0);


pub fn window_size() -> Vector2 {
    Settings::get_mut().window_size.into()
}

// folders
pub const DOWNLOADS_DIR:&str = "downloads";
pub const SONGS_DIR:&str = "songs";
pub const REPLAYS_DIR:&str = "replays";

// database files
pub const SCORE_DATABASE_FILE:&str = "scores.db";

// main fn
fn main() {
    let mut main_benchmark = BenchmarkHelper::new("main");

    // check for missing folders
    check_folder(DOWNLOADS_DIR, true);
    check_folder(REPLAYS_DIR, true);
    check_folder(SONGS_DIR, true);
    check_folder("fonts", false);
    check_folder("audio", false);
    main_benchmark.log("folder check done", true);
    
    let game = Game::new();
    let _ = game.threading.enter();
    main_benchmark.log("game creation complete", true);

    drop(main_benchmark);
    game.game_loop();
}


// helper functions
fn check_folder(dir:&str, create:bool) {
    if !Path::new(dir).exists() {
        if create {
            std::fs::create_dir(dir).expect("error creating folder: ");
        } else {
            panic!("Folder does not exist, but is required: {}", dir);
        }
    }
}

fn check_file(path:&str, download_url:&str) {
    if !Path::new(&path).exists() {
        println!("Check failed for '{}', downloading from '{}'", path, download_url);
        
        let bytes = reqwest::blocking::get(download_url)
            .expect("error with request")
            .bytes()
            .expect("error converting to bytes");

        std::fs::write(path, bytes)
            .expect("Error saving file");
    }
}


/// read a file to the end
fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Lines<BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
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


fn get_file_hash<P:AsRef<Path>>(file_path:P) -> std::io::Result<String> {
    let body = std::fs::read(file_path)?;
    Ok(format!("{:x}", md5::compute(body).to_owned()))
}