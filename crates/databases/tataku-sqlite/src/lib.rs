use tataku_engine::*;
use std::str::FromStr;
use rusqlite::{ Connection, ToSql };
use engine::data;

const MAX_INSERTS_PER_STATEMENT:usize = 1_000;

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


pub struct Database {
    // connection: Arc<Mutex<Connection>>,
    connection: Connection,
}
impl Database {
    // pub fn get<'a>() -> MutexGuard<'a, Connection> {
    //     let now = tataku::Instant::now();
    //     let a = DATABASE.connection.lock();
    //     let duration = now.as_millis();
    //     if duration > 100.0 {info!("db lock took {duration:.4}ms to aquire")};
    //     a
    // }

    pub fn new() -> Self {
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


        // let connection = Arc::new(Mutex::new(connection));


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
        
        // Arc::new(Self {connection})
        Self { connection }
    }

    pub fn insert_or_update(
        &self,
        table_name: &str, 
        operation: &SqlOperation,
        operation_if_failed: Option<&SqlOperation>,
    ) -> Result<(), rusqlite::Error> {
        let mut s:rusqlite::Statement<'_> = self.connection.prepare(&operation.sql).unwrap();
        let values = operation.values
            .iter()
            .map(|i| &**i)
            .collect::<Vec<_>>();

        // if error, probably exists, update instead
        if let Err(e) = s.execute(&*values) {
            let Some(operation) = operation_if_failed 
            else { 
                error!("Failed op {} for table {table_name}: {e}", operation.operation_name);
                return Err(e) 
            };

            self.insert_or_update(table_name, operation, None)?;
        }

        Ok(())
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

    #[allow(clippy::needless_pass_by_value, reason = "Result::map_err(Self::map_err)")]
    fn map_err(e: rusqlite::Error) -> tataku::Error {
        tataku::Error::String(e.to_string())
    }
}
// initializers
impl Database {
    fn init_beatmap_collection(connection: &rusqlite::Connection) {
        // difficulties table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS beatmap_collections (
                beatmap_hash TEXT,
                collection_name TEXT,

                PRIMARY KEY (beatmap_hash, collection_name)
            )", [])
        .expect("error creating db table");
    }
}


impl data::database::BeatmapProvider for Database {
    fn get_beatmaps(&self) -> tataku::Result<Vec<Arc<engine::BeatmapMeta>>> {
        let mut s = self.connection
            .prepare("SELECT * FROM beatmaps")
            .unwrap();
        
        let rows = s
            .query_map([], db_beatmap::row_into_metadata)
            .map_err(Self::map_err)?
            .filter_map(Result::ok)
            .map(Arc::new)
            .collect::<Vec<Arc<engine::BeatmapMeta>>>();

        Ok(rows)
    }

    fn add_beatmaps(&mut self, maps: &[Arc<engine::BeatmapMeta>]) -> tataku::Result<()> {
        let maps_iter = maps
            .chunks(MAX_INSERTS_PER_STATEMENT);

        for map_group in maps_iter {
            let values = map_group
                .iter()
                .flat_map(|a| db_beatmap::beatmap_values(a))
                .collect::<Vec<_>>();
            let values = values.iter()
                .map(|i| &**i)
                .collect::<Vec<_>>();

            let query = db_beatmap::BEATMAP_INSERT.to_string() + &Self::make_values_str(
                db_beatmap::COLUMNS,
                map_group.len()
            );

            self.connection
                .prepare(&query)
                .expect(&query)
                .execute(&*values)
                .map_err(Self::map_err)?;
        }
    
        Ok(())
    }

    fn clear_all_beatmaps(&mut self) -> tataku::Result<()> {
        let statement = "DELETE FROM beatmaps";
        self.connection
            .prepare(statement)
            .expect(statement)
            .execute([])
            .map_err(Self::map_err)?;

        Ok(())
    }


    // ignored
    fn get_ignored_beatmaps(&self) -> tataku::Result<Vec<data::IgnoredBeatmap>> {
        let mut s = self.connection
            .prepare("SELECT * FROM ignore_maps")
            .unwrap();
        
        let list =  s.query_map(
            [], 
            |row| {
                let path = row.get::<_, String>("beatmap_path")?;
                let hash = row.get::<_, String>("beatmap_hash")?;

                let hash_maybe = common::Md5Hash::from_str(&hash);

                match (path.is_empty(), hash.is_empty()) {
                    (true, false) if hash_maybe.is_ok() 
                        => Ok(data::IgnoredBeatmap::Hash(hash_maybe.unwrap())),

                    (false, true) => Ok(data::IgnoredBeatmap::Path(path)),

                    _ => Err(rusqlite::Error::FromSqlConversionFailure(
                        0, 
                        rusqlite::types::Type::Blob, 
                        "ignore angry".into()
                    ))
                }
            }
        )
        .map_err(Self::map_err)?
        .filter_map(|m| {
            if let Err(e) = &m { error!("DB Err: {e}") }
            m.ok()
        })
        .collect::<Vec<_>>();

        Ok(list)
    }

    fn add_ignored_beatmap(
        &mut self, 
        ignored: &data::IgnoredBeatmap
    ) -> tataku::Result<()> {
        const SQL: &str = "INSERT INTO ignore_maps (beatmap_path, beatmap_hash) VALUES ";

        let mut query = SQL.to_string();
        let str = ignored.as_str();
        match ignored {
            data::IgnoredBeatmap::Hash(_) => query += "('', ?)",
            data::IgnoredBeatmap::Path(_) => query += "(?, '')",
        }

        self.connection
            .prepare(&query)
            .expect(&query)
            .execute([&*str])
            .map_err(Self::map_err)?;

        Ok(())
    }

    fn remove_ignored_beatmap(
        &mut self, 
        ignored: &data::IgnoredBeatmap,
    ) -> tataku::Result<()> {
        let column = match ignored {
            data::IgnoredBeatmap::Hash(_) => "beatmap_hash",
            data::IgnoredBeatmap::Path(_) => "beatmap_path",
        };
        let query = format!("dELETE FROM ignore_maps WHERE {column} = ?1");

        let value = ignored.as_str();
        self.connection
            .prepare(&query)
            .expect(&query)
            .execute([&*value])
            .map_err(Self::map_err)?;

        Ok(())
    }



    // collections
    fn get_beatmap_collections(&self) -> tataku::Result<Vec<data::BeatmapCollection>> {
        // helper struct
        struct BeatmapCollectionEntry {
            name: String,
            beatmap: String
        }

        let query = "SELECT * FROM beatmap_collections";
        let mut s = self.connection.prepare(query).expect(query);
        let rows = s.query_map(
            [], 
            |row| Ok(BeatmapCollectionEntry {
                name: row.get("collection_name")?,
                beatmap: row.get("beatmap_hash")?,
            })
        )
        .map_err(Self::map_err)?;

        let mut map = HashMap::<_, Vec<_>>::new();
        for i in rows.filter_map(Result::ok) {
            map.entry(i.name)
                .or_default()
                .push(common::Md5Hash::from_str(&i.beatmap).unwrap());
        }

        let list = map
            .into_iter()
            .map(|(name, beatmaps)| data::BeatmapCollection {
                name,
                beatmaps,
            }).collect();
        Ok(list)
    }

    fn update_beatmap_collection(
        &mut self, 
        _collection: &data::BeatmapCollection
    ) -> tataku::Result<()> {
        todo!("pain and suffering");
    }

    fn add_beatmap_collection(
        &mut self, 
        collection: &data::BeatmapCollection,
    ) -> tataku::Result<()> {
        for chunk in collection.beatmaps.chunks(MAX_INSERTS_PER_STATEMENT) {
            let values = chunk
                .iter()
                .map(common::Md5Hash::to_string)
                .collect::<Vec<_>>();
            
            let values = values
                .iter()
                .flat_map(|a| vec![ collection.name.as_str(), a.as_str() ])
                .collect::<Vec<_>>();

            let values = values.iter()
                .map(|i| i as &dyn ToSql)
                .collect::<Vec<_>>();

            const SQL: &str = "INSERT INTO beatmap_collections (collection_name, beatmap_hash) VALUES ";
            let query = SQL.to_string() + &Self::make_values_str(
                2,
                chunk.len()
            );

            self.connection
                .prepare(&query)
                .expect(&query)
                .execute(&*values)
                .map_err(Self::map_err)?;
        }

        Ok(())
    }

    fn remove_beatmap_collection(&mut self, name: &str) -> tataku::Result<()> {
        const QUERY: &str = "dELETE FROM beatmap_collections WHERE collection_name=?1";

        self.connection
            .prepare(QUERY)
            .expect(QUERY)
            .execute([name])
            .map_err(Self::map_err)?;

        Ok(())
    }
}

impl engine::database::BeatmapPreferencesProvider for Database {
    fn get_beatmap_preferences(
        &self, 
        map: common::Md5Hash,
    ) -> tataku::Result<data::BeatmapPreferences> {
        const QUERY:&str = "SELECT * FROM beatmap_preferences WHERE beatmap_hash=?1";
        self.connection
            .prepare(QUERY)
            .expect(QUERY)
            .query_map(
                [ &map.to_string() ], 
                |row| Ok(data::BeatmapPreferences {
                    audio_offset: row.get("audio_offset")?,
                    background_video: row.get("background_video")?,
                    beatmap_skin: row.get("beatmap_skin").unwrap_or_default(),
                    storyboard: row.get("storyboard")?,
                }),
            )
            .map_err(Self::map_err)?
            .find_map(Result::ok)
            .ok_or_else(|| tataku::Error::String("no row?".into()))
    }
    fn set_beatmap_preferences(
        &mut self, 
        map: common::Md5Hash,
        prefs: &engine::data::BeatmapPreferences,
    ) -> tataku::Result<()> {
        let engine::data::BeatmapPreferences { 
            audio_offset, 
            background_video, 
            storyboard, 
            beatmap_skin,
        } = prefs;
        let map_hash = Box::new(map.to_string());

        self.insert_or_update(
            "beatmap_preferences", 
            &SqlOperation::new(
                "INSERT INTO beatmap_preferences (
                    beatmap_hash, 
                    audio_offset, 
                    background_video, 
                    storyboard, 
                    beatmap_skin
                ) VALUES (?1, ?2, ?3, ?4, ?5)", 
                "INSERT",
                vec![
                    map_hash.clone().into(), 
                    audio_offset.into(),
                    background_video.into(),
                    storyboard.into(),
                    beatmap_skin.into(),
                ]
            ),
            Some(&SqlOperation::new(
                "UPDATE beatmap_preferences 
                    SET audio_offset=?2, background_video=?3, storyboard=?4, beatmap_skin=?5
                    WHERE beatmap_hash=?1
                ", 
                "UPDATE",
                vec![
                    map_hash.clone().into(), 
                    audio_offset.into(),
                    background_video.into(),
                    storyboard.into(),
                    beatmap_skin.into(),
                ]
            ))
        ).map_err(Self::map_err)
    }


    fn get_beatmap_playmode_preferences(
        &self, 
        map: common::Md5Hash,
        playmode: &str,
    ) -> tataku::Result<engine::data::BeatmapPlaymodePreferences> {
        // let map_hash = map_hash.to_string();

        let query = "SELECT * FROM beatmap_mode_preferences WHERE beatmap_hash=?1 AND playmode=?2";
        let mut s = self.connection.prepare(query).expect(query);
        s.query_map(
            [ &map.to_string(), playmode ], 
            |row| Ok(data::BeatmapPlaymodePreferences {
                scroll_speed: row.get("scroll_speed")?,
            })
        )
        .map_err(Self::map_err)?
        .find_map(Result::ok)
        .ok_or_else(|| tataku::Error::String("no row?".into()))
    }
    fn set_beatmap_playmode_preferences(
        &mut self, 
        map: common::Md5Hash,
        playmode: &str,
        prefs: &data::BeatmapPlaymodePreferences
    ) -> tataku::Result<()> {
        let data::BeatmapPlaymodePreferences { 
            scroll_speed 
        } = prefs;

        let map_hash = Box::new(map.to_string());

        self.insert_or_update(
            "beatmap_preferences", 
            &SqlOperation::new(
                "INSERT INTO beatmap_mode_preferences (beatmap_hash, playmode, scroll_speed) VALUES (?1, ?2, ?3, ?4, ?5)", 
                "INSERT",
                vec![
                    (&map_hash).into(), 
                    (&playmode).into(),
                    scroll_speed.into(),
                ]
            ),
            Some(&SqlOperation::new(
                "UPDATE beatmap_mode_preferences 
                    SET scroll_speed=?3, 
                    WHERE beatmap_hash=?1 AND playmode=?2", 
                "UPDATE",
                vec![
                    (&map_hash).into(), 
                    (&playmode).into(),
                    scroll_speed.into(),
                ]
            ))
        ).map_err(Self::map_err)
    }

}

impl engine::database::ScoreProvider for Database {
    fn get_scores(
        &self, 
        map: common::Md5Hash, 
        playmode: &str,
        infos: &tataku_engine::gameplay::GamemodeInfos,
    ) -> tataku::Result<Vec<common::Score>> {
        let mut s = self.connection
            .prepare("SELECT * FROM scores WHERE map_hash=? AND playmode=?")
            .unwrap();
        
        let list = s.query_map(
            [&map.to_string(), playmode], 
            |r| db_score::map_row(r, playmode, infos)
        )
        .map_err(Self::map_err)?
        .filter_map(|m| {
            if let Err(e) = &m {
                error!("score error: {e}");
            }
            m.ok()
        })
        .collect::<Vec<common::Score>>();

        Ok(list)
    }

    fn add_score(&mut self, s: &common::Score) -> tataku::Result<()> {
        trace!("saving score");

        let sql = "INSERT INTO scores (
            map_hash, score_hash,
            username, playmode, time,
            score,
            combo, max_combo,
            accuracy,
            x50, x100, x300, geki, katu, xmiss,
            speed, 
            version,
            mods_string,
            judgments
        ) VALUES (
            ?, ?,
            ?, ?, ?,
            ?,
            ?, ?,
            ?,
            0, 0, 0, 0, 0, 0,
            ?,
            ?,
            ?,
            ?
        )";
        let params: &[&(dyn rusqlite::ToSql + Send + Sync); 13] = &[
            &s.beatmap_hash.to_string(), &s.hash(),
            &s.username, &s.playmode, &s.time,
            &s.score,
            &s.combo, &s.max_combo,
            &s.accuracy,
            // s.x50, s.x100, s.x300, s.xgeki, s.xkatu, s.xmiss, 
            &s.speed.as_u16(),
            &s.version,
            &s.mods_string_sorted(),
            &s.judgment_string()
        ];

        self.connection
            .prepare(sql)
            .expect(sql)
            .execute(params)
            .map_err(Self::map_err)?;
        Ok(())
    }
}

impl engine::database::DatabaseProvider for Database {}

mod db_beatmap {
    use super::*;
    use engine::BeatmapMeta;

    pub const COLUMNS: usize = 20;
    pub const BEATMAP_INSERT: &str = "
    INSERT INTO beatmaps (
        beatmap_path, beatmap_hash, beatmap_type,

        playmode, 
        artist, artist_unicode,
        title, title_unicode,
        creator, version,

        audio_filename, image_filename,
        audio_preview, duration,
        
        hp, od, cs, ar,
        
        bpm_min, bpm_max
    ) VALUES ";

    pub fn row_into_metadata(r: &rusqlite::Row) -> rusqlite::Result<BeatmapMeta> {
        fn arcstr(r: &rusqlite::Row, idx: &str) -> rusqlite::Result<ArcStr> {
            Ok(r.get::<_, String>(idx)?.into())
        }

        Ok(BeatmapMeta {
            file_path: arcstr(r, "beatmap_path")?,
            beatmap_hash: r.get::<_, String>("beatmap_hash")?.try_into().unwrap(),
            beatmap_type: r.get::<_, u8>("beatmap_type")?.into(),
            mode: arcstr(r, "playmode")?,
            artist: arcstr(r, "artist")?,
            title: arcstr(r, "title")?,
            artist_unicode: arcstr(r, "artist_unicode")?,
            title_unicode: arcstr(r, "title_unicode")?,
            creator: arcstr(r, "creator")?,
            version: arcstr(r, "version")?,
            audio_filename: arcstr(r, "audio_filename")?,
            image_filename: arcstr(r, "image_filename")?,
            audio_preview: r.get("audio_preview")?,
            hp: r.get("hp")?,
            od: r.get("od")?,
            ar: r.get("ar")?,
            cs: r.get("cs")?,

            duration: r.get("duration")?,
            bpm_min: r.get("bpm_min").unwrap_or(0.0),
            bpm_max: r.get("bpm_max").unwrap_or(0.0),
        })
    }

    pub fn beatmap_values(map: &BeatmapMeta) -> Vec<SqlValue<'_>> {
        let mut bpm_min = Box::new(map.bpm_min);
        let mut bpm_max = Box::new(map.bpm_max);
        if !bpm_min.is_normal() {
            *bpm_min = 0.0;
        }
        if !bpm_max.is_normal() {
            *bpm_max = 99999999.0;
        }
        let beatmap_type = Box::new(u8::from(map.beatmap_type));
        let hash = Box::new(map.beatmap_hash.to_string());

        #[allow(clippy::box_collection, reason = "SqlValue::Owned(Box<_>)")]
        fn arcstr(s: &ArcStr) -> Box<String> {
            Box::new(s.to_string())
        }
        
        vec![
            arcstr(&map.file_path).into(), hash.into(), beatmap_type.into(),

            arcstr(&map.mode).into(),
            arcstr(&map.artist).into(), arcstr(&map.artist_unicode).into(),
            arcstr(&map.title).into(), arcstr(&map.title_unicode).into(),
            arcstr(&map.creator).into(), arcstr(&map.version).into(),
            
            arcstr(&map.audio_filename).into(), arcstr(&map.image_filename).into(),
            (&map.audio_preview).into(), (&map.duration).into(),

            (&map.hp).into(), (&map.od).into(), (&map.cs).into(), (&map.ar).into(),

            bpm_min.into(), bpm_max.into()
        ]
    }

}

mod db_score {
    use tataku_engine::*;

    pub fn map_row(
        r: &rusqlite::Row,
        playmode: &str,
        infos: &tataku_engine::gameplay::GamemodeInfos,
    ) -> Result<common::Score, rusqlite::Error> {
        let _score_hash:String = r.get("score_hash")?;

        let mut mods_string:Option<String> = r.get("mods_string").ok();
        if let Some(str) = &mods_string
        && str.is_empty() {
            mods_string = None;
        }

        let mut judgments = HashMap::new();

        // string will be key:val|key:val
        let judgment_str = r
            .get::<_, String>("judgments")
            .ok()
            .and_then(|s| (!s.is_empty()).then_some(s));
        if let Some(judgment_string) = judgment_str {
            judgments = common::Score::judgments_from_string(&judgment_string);
        } else { // no judgments, load legacy values
            for key in [
                "x50",
                "x100",
                "x300",
                "xmiss",
                "xgeki",
                "xkatu"
            ] {
                let val = r.get(key).unwrap_or_default();
                judgments.insert(key.to_owned(), val);
            }
        }

        let mut score = common::Score {
            version: r.get("version").unwrap_or(1), // v1 didnt include version in the table
            username: r.get("username")?,
            playmode: r.get("playmode")?,
            time: r.get("time").unwrap_or(0),
            score: r.get("score")?,
            combo: r.get("combo")?,
            max_combo: r.get("max_combo")?,
            accuracy: r.get("accuracy").unwrap_or_default(),
            beatmap_hash: r.get::<_, String>("map_hash")?.try_into().unwrap(),
            speed: r.get::<_, f32>("speed").map(common::GameSpeed::from_f32).unwrap_or_default(),
            hit_timings: Vec::new(),
            judgments,
            ..Default::default()
        };


        // this is bad but its fineee
        if let Some((mods_string, info)) = mods_string.zip(infos.get_info(playmode).ok()) {
            let mods = mods_string.split("|");
            let all_mods = engine::gameplay::mods::ModManager::mods_for_playmode_as_hashmap(info);
            for m in mods {
                let Some(m) = all_mods.get(m) else { continue };
                score.mods.push((*m).into());
            }
        }

        Ok(score)
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
