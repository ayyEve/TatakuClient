use crate::prelude::*;

// lazy_static::lazy_static! {
//     static ref DIFF_CACHE:HashMap<>
// }


impl Database {
    pub async fn insert_many_diffs(playmode:&PlayMode, mods:&ModManager, diffs:impl Iterator<Item=(String, f32)>) {
        let mods = serde_json::to_string(mods).unwrap().replace("'", "\\'");
        
        let mut insert_query = "INSERT INTO difficulties (beatmap_hash, playmode, mods_string, diff_calc_version, diff) VALUES ".to_owned();
        let mut hash_list = Vec::new();

        let version = 1;

        for (hash, diff) in diffs {
            hash_list.push(format!("'{hash}'"));
            insert_query += &format!("('{hash}', '{playmode}', '{mods}', {version}, {diff}),")
        }
        let insert_query = insert_query.trim_end_matches(",");

        // delete existing entries
        let hash_list = hash_list.join(",");
        let delete_query = format!("DELETE FROM difficulties WHERE beatmap_hash IN ({hash_list}) AND playmode='{playmode}' AND mods_string='{mods}'");

        let db = Self::get().await;
        let mut s = db.prepare(&delete_query).unwrap();
        if let Err(e) = s.execute([]) {
            error!("Error deleting from difficulties table: {e}")
        }

        // insert new vals
        let mut s = db.prepare(&insert_query).unwrap();
        if let Err(e) = s.execute([]) {
            error!("Error inserting into difficulties table: {e}")
        }

    }

    pub async fn insert_diff(map_hash:&String, playmode:&PlayMode, mods:&ModManager, diff:f32) {
        let db = Self::get().await;

        let mods = serde_json::to_string(mods).unwrap().replace("'", "\\'");

        let diff_calc_version = 1;

        let sql = format!(
            "INSERT INTO difficulties (
                beatmap_hash, 
                playmode,
                mods_string,
                diff_calc_version,
                diff
            ) VALUES (
                '{map_hash}',
                '{playmode}',
                '{mods}',
                {diff_calc_version},
                {diff}
            )"
        );
        let mut s = db.prepare(&sql).unwrap();

        // error is entry already exists
        if let Err(_) = s.execute([]) {
            // trance!("updating diff: {diff}");
            let sql = format!(
                "UPDATE difficulties SET diff={} WHERE beatmap_hash='{}' AND playmode='{}' AND mods_string='{}'", 
                diff,
                map_hash,
                playmode,
                mods,
            );
            let mut s = db.prepare(&sql).unwrap();

            if let Err(e) = s.execute([]) {
                error!("Error inserting/updateing difficulties table: {e}")
            }
        }
    }


    pub async fn get_all_diffs(playmode: &PlayMode, mods: &ModManager) -> HashMap<String, f32> {
        let db = Self::get().await;
        let mods = serde_json::to_string(mods).unwrap().replace("'", "\\'");

        let query = format!(
            "SELECT beatmap_hash, diff FROM difficulties WHERE playmode='{}' AND mods_string='{}'",
            playmode,
            mods
        );
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], |row| Ok((
            row.get::<&str, String>("beatmap_hash")?,
            row.get::<&str, f32>("diff")?
        )));
        
        let mut map = HashMap::new();

        if let Ok(rows) = res {
            for (map_hash, diff) in rows.filter_map(|r|r.ok()) {
                map.insert(map_hash, diff);
            }
        }

        map
    }

    pub async fn get_diff(map_hash: &String, playmode: &PlayMode, mods: &ModManager) -> Option<f32> {
        let db = Self::get().await;
        let mods = serde_json::to_string(mods).unwrap().replace("'", "\\'");

        let query = format!(
            "SELECT diff FROM difficulties WHERE beatmap_hash='{}' AND playmode='{}' AND mods_string='{}'",
            map_hash,
            playmode,
            mods
        );
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], |row| row.get::<&str, f32>("diff"));

        if let Ok(mut rows) = res {
            rows.find_map(|r|r.ok())
        } else {
            None
        }
    }
}


// #[derive(Clone)]
// pub struct DifficultyEntry {
//     pub beatmap_hash: String,
//     pub playmode: String,
//     pub mods_string: String,
//     pub diff_calc_version: u16,
//     pub diff: f64
// }
// impl DifficultyEntry {
//     pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
//         Ok(Self {
//             beatmap_hash: row.get("beatmap_hash")?,
//             playmode: row.get("playmode")?,
//             mods_string: row.get("mods_string")?,
//             diff_calc_version: row.get("diff_calc_version")?,
//             diff: row.get("diff")?,
//         })
//     }
// }