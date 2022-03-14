use crate::prelude::*;
use rusqlite::Connection;

mod difficulty_database;
pub use difficulty_database::*;

mod score_database;
pub use score_database::*;


lazy_static::lazy_static! {
    pub static ref DATABASE: Arc<Mutex<Connection>> = {
        let db = Connection::open("tataku.db").unwrap();
        // scores table
        db.execute(
            "CREATE TABLE IF NOT EXISTS scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT,
                map_hash TEXT,
                score_hash TEXT,
                playmode INTEGER,
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
        db.execute(
            "CREATE TABLE IF NOT EXISTS beatmaps (
                beatmap_path TEXT,
                beatmap_hash TEXT,

                playmode INTEGER,
                beatmap_version INTEGER,
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
                ar REAL,
                
                slider_multiplier REAL,
                slider_tick_rate REAL
            )", [])
        .expect("error creating db table");

        add_new_entries(&db);

        Arc::new(Mutex::new(db))
    };
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
];

fn add_new_entries(db: &Connection) {
    // score entries
    for (col, t) in SCORE_ENTRIES {
        match db.execute(&format!("ALTER TABLE scores ADD {} {};", col, t), []) {
            Ok(_) => println!("Column added to scores db: {}", col),
            Err(e) => println!("Error adding column to scores db: {}", e),
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
