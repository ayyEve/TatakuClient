use crate::prelude::*;
use rusqlite::Connection;

mod beatmaps;
mod score_database;
mod beatmap_preferences;
mod difficulty_database;

pub use beatmaps::*;
pub use score_database::*;
pub use beatmap_preferences::*;
pub use difficulty_database::*;



lazy_static::lazy_static! {
    pub static ref DATABASE: Arc<Database> = Database::new();
}

// add new db columns here
// needed to add new cols to existing dbs
// this is essentially migrations, but a lazy way to do it lol
const SCORE_ENTRIES: &[(&str, &str)] = &[
    ("x50", "INTEGER"),
    ("katu", "INTEGER"),
    ("geki", "INTEGER"),
    ("speed", "INTEGER"),
    ("version", "INTEGER"),
    ("mods_string", "TEXT"),
];
const BEATMAP_ENTRIES: &[(&str, &str)] = &[
    ("bpm_min", "INTEGER"),
    ("bpm_max", "INTEGER"),
    ("beatmap_type", "INTEGER"),
];

fn add_new_entries(db: &Connection) {
    // score entries
    for (col, t) in SCORE_ENTRIES {
        match db.execute(&format!("ALTER TABLE scores ADD {} {};", col, t), []) {
            Ok(_) => println!("Column added to scores db: {}", col),
            Err(e) => {
                let e = format!("{}", e);
                // only log error if its not a duplicate column name
                if !e.contains("duplicate column name") {
                    println!("Error adding column to scores db: {}", e)
                }
            },
        }
    }
    
    // beatmap entries
    for (col, t) in BEATMAP_ENTRIES {
        match db.execute(&format!("ALTER TABLE beatmaps ADD {} {};", col, t), []) {
            Ok(_) => println!("Column added to beatmaps db: {}", col),
            Err(e) => println!("Error adding column to beatmaps db: {}", e),
        }
    }
}


#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}
impl Database {
    pub fn get<'a>() -> MutexGuard<'a, Connection> {DATABASE.connection.lock()}

    fn new() -> Arc<Self> {
        let connection = Connection::open("tataku.db").unwrap();
        // scores table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT,
                map_hash TEXT,
                score_hash TEXT,
                playmode TEXT,
                score INTEGER,
                combo INTEGER,
                max_combo INTEGER,
                x50 INTEGER,
                x100 INTEGER,
                x300 INTEGER,
                geki INTEGER,
                katu INTEGER,
                xmiss INTEGER,
                speed INTEGER,
                version INTEGER
         )", [])
        .expect("error creating db table");

        // beatmaps table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS beatmaps (
                beatmap_path TEXT,
                beatmap_hash TEXT PRIMARY KEY,

                playmode TEXT,
                artist TEXT,
                title TEXT,
                artist_unicode TEXT,
                title_unicode TEXT,
                creator TEXT,
                version TEXT,

                audio_filename TEXT,
                image_filename TEXT,
                audio_preview REAL,
                
                duration REAL,
                
                hp REAL,
                od REAL,
                cs REAL,
                ar REAL
            )", [])
        .expect("error creating db table");

        // ignore maps table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS ignore_maps (
                beatmap_path TEXT,
                beatmap_hash TEXT,
                
                PRIMARY KEY (beatmap_path, beatmap_hash)
            )", [])
        .expect("error creating db table");


        // beatmap preferences table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS beatmap_preferences (
                beatmap_hash TEXT PRIMARY KEY,
                background_video BOOL,
                storyboard BOOL
            )", [])
        .expect("error creating db table");

        // beatmap mode preferences table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS beatmap_mode_preferences (
                beatmap_hash TEXT,
                playmode TEXT,
                scroll_speed REAL,

                PRIMARY KEY (beatmap_hash, playmode)
            )", [])
        .expect("error creating db table");

        // difficulties table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS difficulties (
                beatmap_hash TEXT,
                playmode TEXT,
                mods_string TEXT,
                diff_calc_version INTEGER,
                diff REAL,

                PRIMARY KEY (beatmap_hash, playmode, mods_string, diff_calc_version)
            )", [])
        .expect("error creating db table");

        add_new_entries(&connection);

        let connection = Arc::new(Mutex::new(connection));
        Arc::new(Self {connection})
    }

}