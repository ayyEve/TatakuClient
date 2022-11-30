use crate::prelude::*;
use rusqlite::Connection;
use tokio::sync::mpsc::{channel, Sender};

lazy_static::lazy_static! {
    pub static ref DATABASE: Arc<Database> = Database::new();
    pub static ref DATABASE_OPERATIONS_QUEUE:OnceCell<Sender<DatabaseQuery>> = OnceCell::const_new();
}

// add new db columns here
// needed to add new cols to existing dbs
// this is essentially migrations, but a lazy way to do it lol
const MIGRATIONS:&[(&str, &[(&str, &str)])] = &[
    ("scores", &[
        ("x50", "INTEGER"),
        ("katu", "INTEGER"),
        ("geki", "INTEGER"),
        ("speed", "REAL"),
        ("accuracy", "REAL"),
        ("version", "INTEGER"),
        ("mods_string", "TEXT"),
        ("judgments", "TEXT"),
        ("time", "INTEGER"),
    ]),
    ("beatmaps", &[
        ("bpm_min", "INTEGER"),
        ("bpm_max", "INTEGER"),
        ("beatmap_type", "INTEGER"),
    ]),
    ("ui_elements", &[
        ("visible", "BOOL"),
        ("window_size_x", "REAL"),
        ("window_size_y", "REAL"),
    ]),
    ("beatmap_preferences", &[
        ("audio_offset", "REAL")
    ])
];

fn perform_migrations(db: &Connection) {
    for (table, entries) in MIGRATIONS {
        for (col, t) in *entries {
            match db.execute(&format!("ALTER TABLE {table} ADD {col} {t};"), []) {
                Ok(_) => debug!("Column added to {table} db: {col}"),
                Err(e) => {
                    let e = format!("{}", e);
                    // only log error if its not a duplicate column name
                    if !e.contains("duplicate column name") {
                        error!("Error adding column to scores db: {}", e)
                    }
                }
            }
        }
    }
}


#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}
impl Database {
    pub async fn get<'a>() -> tokio::sync::MutexGuard<'a, Connection> {
        let now = Instant::now();
        let a = DATABASE.connection.lock().await;
        let duration = now.elapsed().as_secs_f32() * 1000.0;
        if duration > 100.0 {info!("db lock took {:.4}ms to aquire", duration)};
        a
    }
    // pub async fn get_diffcalc<'a>() -> tokio::sync::MutexGuard<'a, Connection> {
    //     let now = Instant::now();
    //     let a = DATABASE.connection.lock().await;
    //     let duration = now.elapsed().as_secs_f32() * 1000.0;
    //     if duration > 0.5 {info!("diffcalc db lock took {:.4}ms to aquire", duration)};
    //     a
    // }

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
                speed REAL,
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
                audio_offset REAL,
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

        // ui element things table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS ui_elements (
                name TEXT PRIMARY KEY,
                pos_x REAL,
                pos_y REAL,
                scale_x REAL,
                scale_y REAL,
                visible BOOL
            )", [])
        .expect("error creating db table");



        Database::init_beatmap_collection(&connection);

        perform_migrations(&connection);


        let connection = Arc::new(Mutex::new(connection));


        let (sender, mut receiver) = channel(1000);
        if let Err(e) = DATABASE_OPERATIONS_QUEUE.set(sender) {
            panic!("no {}", e)
        }

        // setup operation performer
        tokio::spawn(async move {
            
            while let Some(op) = receiver.recv().await {
                match op {
                    DatabaseQuery::InsertOrUpdate { sql, table_name, operation, sql_if_failed , operation_if_failed} => {
                        let db = Self::get().await;
                        let mut s = db.prepare(&sql).unwrap();
                        let res = s.execute([]);

                        // if error, probably exists, update instead
                        if let Err(e) = res {
                            if let Some(sql) = &sql_if_failed {
                                let mut s = db.prepare(&sql).unwrap();
                                let res = s.execute([]);

                                if let Err(e) = res {
                                    let operation = operation_if_failed.unwrap_or(format!("if_failed({operation})"));
                                    error!("Failed op {operation} for table {table_name}: {e}");
                                }
                            } else {
                                error!("Failed op {operation} for table {table_name}: {e}");
                            }
                        }
                    },
                }
            }
        });
        
        Arc::new(Self {connection})
    }

    pub fn add_query(q: DatabaseQuery) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = DATABASE_OPERATIONS_QUEUE.get().unwrap().send(q).await {
                    error!("error sending database query! {}", e);
                }
            });
        });
    }
}


pub enum DatabaseQuery {
    InsertOrUpdate { sql: String, table_name: String, operation: String, sql_if_failed: Option<String>, operation_if_failed: Option<String> }
    // Other {sql: String, on_complete: MultiBomb<>}
}