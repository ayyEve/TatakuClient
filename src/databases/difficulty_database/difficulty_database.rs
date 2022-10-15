use crate::prelude::*;
use rusqlite::Connection;

lazy_static::lazy_static! {
    pub static ref DIFFICULTY_DATABASE: Arc<DifficultyDatabase> = DifficultyDatabase::new();
}

#[derive(Clone)]
pub struct DifficultyDatabase {
    connection: Arc<Mutex<Connection>>,
}


impl DifficultyDatabase {
    pub async fn get<'a>() -> tokio::sync::MutexGuard<'a, Connection> {
        let now = Instant::now();
        let a = DIFFICULTY_DATABASE.connection.lock().await;
        let duration = now.elapsed().as_secs_f32() * 1000.0;
        if duration > 5.0 { info!("diff db lock took {:.4}ms to aquire", duration) };
        a
    }


    pub fn new() -> Arc<Self> {
        let connection = Connection::open("tataku_diffs.db").unwrap();

        // difficulties table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS difficulties (
                map INTEGER,
                mode INTEGER,
                speed INTEGER,
                diff REAL,

                PRIMARY KEY (map, mode, speed)
            )", [])
        .expect("error creating db table");

        // modes map table (mode string -> integer)
        connection.execute(
            "CREATE TABLE IF NOT EXISTS mode_map (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode TEXT UNIQUE
            )", [])
        .expect("error creating db table");

        // beatmap_hash
        connection.execute(
            "CREATE TABLE IF NOT EXISTS beatmap_hash_map (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hash TEXT UNIQUE
            )", [])
        .expect("error creating db table");

        // update any tables that are missing values
        perform_migrations(&connection);


        let connection = Arc::new(Mutex::new(connection));
        Arc::new(Self {connection})
    }



    pub async fn insert_many_diffs(entries: &HashMap<DifficultyEntry, f32>) {
        if entries.len() == 0 { return }
        info!("insert many");
        
        let mut insert_query = "INSERT OR IGNORE INTO difficulties (map, mode, speed, diff) VALUES ".to_owned();
        let mut hash_list = Vec::new();
        let playmode = &entries.keys().next().unwrap().playmode;

        for (entry, diff) in entries {
            let hash = &entry.map_hash;
            let speed = entry.mods.speed;

            insert_query += &format!("((SELECT id FROM beatmap_hash_map WHERE hash='{hash}'), (SELECT mode FROM mode_t), {speed}, {diff}),");
            hash_list.push(hash);
        }

        let insert_query = insert_query.trim_end_matches(",");
        let maps = verify_beatmap_hashes(&hash_list);
        let mode = verify_mode(playmode);

        let with = format!(
            "WITH mode_t AS (SELECT id as mode FROM mode_map WHERE mode='{playmode}')",
        );

        let sql = [
            maps,
            mode.insert_query.clone(), 
            with + insert_query
        ].join(";\n");

        std::fs::write("insert.sql", &sql).unwrap();

        info!("inserting {} diffs into db", hash_list.len());

        let db = Self::get().await;
        if let Err(e) = db.execute_batch(&sql) {
            info!("error inserting into diff db: {e}")
        }
        info!("insert done");

    }


    pub async fn get_all_diffs() -> HashMap<DifficultyEntry, f32> {
        info!("retreive many");

        let query = "SELECT d.diff, d.speed, b.hash, m.mode FROM difficulties d JOIN beatmap_hash_map b ON (d.map = b.id) JOIN mode_map m ON (d.mode = m.id)";

        
        let db = Self::get().await;
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], |row| Ok((
            row.get::<&str, String>("hash")?,
            row.get::<&str, String>("mode")?,
            row.get::<&str, u16>("speed")?,
            row.get::<&str, f32>("diff")?
        )));
        info!("query done");
        
        let mut map = HashMap::new();

        if let Ok(rows) = res {
            for (map_hash, mode, speed, diff) in rows.filter_map(|r|r.ok()) {
                let mut mods = ModManager::default();
                mods.speed = speed;

                let k = DifficultyEntry::new(map_hash, mode, mods);
                map.insert(k, diff);
            }
        }
        
        info!("map done");
        map
    }

}




// add new db columns here
// needed to add new cols to existing dbs
// this is essentially migrations, but a lazy way to do it lol
const MIGRATIONS:&[(&str, &[(&str, &str)])] = &[
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



fn verify(table_name: &str, col_name: &str, item: &String) -> DbVerify {
    DbVerify {
        insert_query: format!("INSERT OR IGNORE INTO {table_name}({col_name}) VALUES ('{item}')"),
        select_query: format!("SELECT id AS col_name FROM {table_name} WHERE {col_name}='{item}'")
    }
}
fn verify_beatmap_hashes(hashes:&Vec<&String>) -> String {
    // INSERT OR IGNORE INTO beatmap_hash_map (beatmap_hash) VALUES ('2'), ('3')
    format!(
        "INSERT OR IGNORE INTO beatmap_hash_map (hash) VALUES {}",
        hashes.iter().map(|hash|format!("('{hash}')")).collect::<Vec<String>>().join(",")
    )
}
fn verify_mode(mode:&PlayMode) -> DbVerify {
    verify("mode_map", "mode", mode)
}
struct DbVerify {
    insert_query: String,
    select_query: String
}


#[tokio::test]
async fn test_diff_db() {
    let mode = "osu".to_owned();
    let mods = ModManager::new();

    let mut diffs = HashMap::new();
    diffs.insert(DifficultyEntry::new("1".to_owned(), mode.clone(), mods.clone()), 1.0);
    diffs.insert(DifficultyEntry::new("2".to_owned(), mode.clone(), mods.clone()), 2.0);
    diffs.insert(DifficultyEntry::new("3".to_owned(), mode.clone(), mods.clone()), 3.0);


    DifficultyDatabase::insert_many_diffs(&diffs).await;
    println!("insert test done");

    let diffs = DifficultyDatabase::get_all_diffs().await;
    println!("retreive test done: {}", diffs.len());
}
