// TODO: use a provider to save/load scores/replays etc

use crate::prelude::*;
use rusqlite::{ Connection, ToSql };
// use tokio::sync::mpsc::{ channel, Sender };


lazy_static::lazy_static! {
    pub static ref DATABASE: Arc<Database> = Database::new();
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
                    let e = e.to_string();
                    // only log error if its not a duplicate column name
                    if !e.contains("duplicate column name") {
                        error!("Error adding column to scores db: {e}");
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
    pub fn get<'a>() -> MutexGuard<'a, Connection> {
        let now = tataku::Instant::now();
        let a = DATABASE.connection.lock();
        let duration = now.as_millis();
        if duration > 100.0 {info!("db lock took {duration:.4}ms to aquire")};
        a
    }

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


        // let (sender, mut receiver) = channel(1000);
        // DATABASE_OPERATIONS_QUEUE.set(sender).unwrap();

        // // setup operation performer
        // tokio::spawn(async move {
            
        //     while let Some(op) = receiver.recv().await {
        //         match op {
        //             DatabaseQuery::InsertOrUpdate { sql, table_name, operation, sql_if_failed , operation_if_failed } => {
                        
        //             },
        //         }
        //     }
        // });
        
        Arc::new(Self {connection})
    }

    pub fn insert_or_update(
        table_name: &str, 
        operation: &SqlOperation,
        operation_if_failed: Option<&SqlOperation>,
    ) {
        let db = Self::get();
        let mut s:rusqlite::Statement<'_> = db.prepare(&operation.sql).unwrap();
        let values = operation.values
            .iter()
            .map(|i| &**i)
            .collect::<Vec<_>>();

        let res = s.execute(&*values);

        // if error, probably exists, update instead
        if let Err(e) = res {
            if let Some(operation) = operation_if_failed {
                Self::insert_or_update(table_name, operation, None);
            } else {
                error!("Failed op {} for table {table_name}: {e}", operation.operation_name);
            }
        }
    }


    pub fn make_values_str(columns: usize, entries: usize) -> String {
        let mut index = 1;

        (0..entries)
            .map(|_| (0..columns)
                .map(|_| { index += 1; format!("?{}", index - 1) })
                .collect::<Vec<_>>()
                .join(", ")
            )
            .map(|v| format!("({v})"))
            .collect::<Vec<_>>()
            .join(",\n")
    }

}

pub struct SqlOperation<'a> {
    pub sql: String,
    pub operation_name: String,
    pub values: Vec<SqlValue<'a>>, 
}
impl<'a> SqlOperation<'a> {
    pub fn new(
        sql: impl Into<String>,
        operation_name: impl Into<String>,
        values: Vec<SqlValue<'a>>,
    ) -> Self {
        Self {
            sql: sql.into(),
            operation_name: operation_name.into(),
            values,
        }
    }
}

pub enum SqlValue<'a> {
    Owned(Box<dyn ToSql>),
    Borrowed(&'a dyn ToSql),
}
impl<T: ToSql + 'static> From<Box<T>> for SqlValue<'_> {
    fn from(value: Box<T>) -> Self {
        Self::Owned(value)
    }
}
impl<'a, T: ToSql> From<&'a T> for SqlValue<'a> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}
impl<'a> Deref for SqlValue<'a> {
    type Target = dyn ToSql + 'a;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(b) => b.as_ref(),
            Self::Borrowed(b) => *b,
        }
    }
}



#[test]
fn test_make_values_str() {
    assert_eq!(
        "(?1, ?2, ?3, ?4),\n(?5, ?6, ?7, ?8),\n(?9, ?10, ?11, ?12)", 
        Database::make_values_str(4, 3)
    );
}
