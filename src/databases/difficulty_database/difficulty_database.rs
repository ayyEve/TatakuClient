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
        if duration > 0.5 {info!("diff db lock took {:.4}ms to aquire", duration)};
        a
    }


    pub fn new() -> Arc<Self> {
        let connection = Connection::open("tataku_diffs.db").unwrap();

        // difficulties table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS difficulties (
                map INTEGER,
                mode INTEGER,
                mods INTEGER,
                diff_calc_version INTEGER,
                diff REAL,

                PRIMARY KEY (map, mode, mods, diff_calc_version)
            )", [])
        .expect("error creating db table");

        // modes map table (mode string -> integer)
        connection.execute(
            "CREATE TABLE IF NOT EXISTS mode_map (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode TEXT UNIQUE
            )", [])
        .expect("error creating db table");

        // mods
        connection.execute(
            "CREATE TABLE IF NOT EXISTS mods_map (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mods TEXT UNIQUE
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



    pub async fn insert_many_diffs(playmode:&PlayMode, mods:&ModManager, diffs:impl Iterator<Item=(String, f32)>) {
        info!("insert many");
        let version = 1;

        let mods_str = serde_json::to_string(mods).unwrap().replace("'", "\\'");
        
        let mut insert_query = "INSERT OR IGNORE INTO difficulties (map, mode, mods, diff_calc_version, diff) VALUES ".to_owned();
        let mut hash_list = Vec::new();


        for (hash, diff) in diffs {
            insert_query += &format!("((SELECT id FROM beatmap_hash_map WHERE hash='{hash}'), (SELECT mode FROM diff_info), (SELECT mod FROM diff_info), {version}, {diff}),");
            hash_list.push(hash);
        }

        let insert_query = insert_query.trim_end_matches(",");
        let maps = verify_beatmap_hashes(&hash_list);
        let mods = verify_mods(&mods_str);
        let mode = verify_mode(playmode);

        let with = format!(
            "WITH 
                mods_t AS (SELECT id as mod FROM mods_map WHERE mods='{mods_str}'),
                mode_t AS (SELECT id as mode FROM mode_map WHERE mode='{playmode}'),
                diff_info AS (select * from mods_t JOIN mode_t)",
        );

        let sql = [
            maps, 
            mods.insert_query.clone(), 
            mode.insert_query.clone(), 
            with + insert_query
        ].join(";\n");

        info!("inserting {} diffs into db", hash_list.len());

        let db = Self::get().await;
        if let Err(e) = db.execute_batch(&sql) {
            warn!("error inserting into diff db: {e}")
        }
        info!("insert done");

        // let delete_query = format!("DELETE FROM difficulties WHERE beatmap_hash IN ({hash_list}) AND playmode='{playmode}' AND mods_string='{mods}'");
        // let mut s = db.prepare(&delete_query).unwrap();
        // if let Err(e) = s.execute([]) {
        //     error!("Error deleting from difficulties table: {e}")
        // }

        // // insert new vals
        // let mut s = db.prepare(&insert_query).unwrap();
        // if let Err(e) = s.execute([]) {
        //     error!("Error inserting into difficulties table: {e}")
        // }

    }


    pub async fn get_all_diffs(playmode: &PlayMode, mods: &ModManager) -> HashMap<String, f32> {
        info!("retreive many");

        let mods = serde_json::to_string(mods).unwrap().replace("'", "\\'");

        let mode = verify_mode(playmode);
        let mods = verify_mods(&mods);

        let query = format!(
            "SELECT d.diff, b.hash FROM difficulties d JOIN beatmap_hash_map b ON (d.map = b.id) WHERE d.mode=({}) AND d.mods=({})",
            mode.select_query,
            mods.select_query
        );

        
        let db = Self::get().await;
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], |row| Ok((
            row.get::<&str, String>("hash")?,
            row.get::<&str, f32>("diff")?
        )));
        info!("query done");
        
        let mut map = HashMap::new();

        if let Ok(rows) = res {
            for (map_hash, diff) in rows.filter_map(|r|r.ok()) {
                map.insert(map_hash, diff);
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
fn verify_beatmap_hashes(hashes:&Vec<String>) -> String {
    // INSERT OR IGNORE INTO beatmap_hash_map (beatmap_hash) VALUES ('2'), ('3')
    format!(
        "INSERT OR IGNORE INTO beatmap_hash_map (hash) VALUES {}",
        hashes.iter().map(|hash|format!("('{hash}')")).collect::<Vec<String>>().join(",")
    )
}
fn verify_mods(mods_str:&String) -> DbVerify {
    verify("mods_map", "mods", mods_str)
}
fn verify_mode(mode:&PlayMode) -> DbVerify {
    verify("mode_map", "mode", mode)
}
struct DbVerify {
    insert_query: String,
    select_query: String
}


#[test]
fn test_diff_db() {
    let runtime = tokio::runtime::Builder::new_current_thread().build().expect("no runtime?");

    runtime.block_on(async {
        let mode = "osu".to_owned();
        let mods = ModManager::new();

        let diffs = vec![
            ("1".to_owned(), 0.1),
            ("2".to_owned(), 0.2),
            ("3".to_owned(), 0.3),
            ("4".to_owned(), 0.4),
        ];

        DifficultyDatabase::insert_many_diffs(&mode, &mods, diffs.iter().map(|i|i.clone())).await;

        println!("insert test done");

        let diffs = DifficultyDatabase::get_all_diffs(&mode, &mods).await;

        
        println!("retreive test done: {}", diffs.len());
    });
}



/*
WITH 
	mods_t AS (SELECT id as mod FROM mods_map WHERE mods='{"speed":1.0,"easy":false,"hard_rock":false,"autoplay":false,"nofail":false}'),
    mode_t AS (SELECT id as mode FROM mode_map WHERE mode='osu'),
    
    diff_info AS (select * from mods_t JOIN mode_t)


INSERT INTO difficulties (map, mode, mods, diff_calc_version, diff) VALUES 
((SELECT id FROM beatmap_hash_map WHERE hash='1'), (select mode from diff_info), (select mod from diff_info), 1, 0.1),
((SELECT id FROM beatmap_hash_map WHERE hash='2'), (select mode from diff_info), (select mod from diff_info), 1, 0.2),
((SELECT id FROM beatmap_hash_map WHERE hash='3'), (select mode from diff_info), (select mod from diff_info), 1, 0.3),
((SELECT id FROM beatmap_hash_map WHERE hash='4'), (select mode from diff_info), (select mod from diff_info), 1, 0.4)
 */