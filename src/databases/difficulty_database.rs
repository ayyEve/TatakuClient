use crate::prelude::*;

impl Database {
    pub fn insert_diff(map_hash:&String, playmode:&PlayMode, mods:&ModManager, diff:f32) {
        let db = Self::get();

        let mods = serde_json::to_string(mods).unwrap().replace("'", "\\'");

        let sql = format!(
            "INSERT INTO difficulties (
                beatmap_hash, 
                playmode,
                mods_string,
                diff_calc_version,
                diff
            ) VALUES (
                '{}',
                '{}',
                '{}',
                {},
                {}
            )", 
            map_hash,
            playmode,
            mods,
            1,
            diff
        );
        let mut s = db.prepare(&sql).unwrap();

        // error is entry already exists
        if let Err(_) = s.execute([]) {
            // println!("updating diff: {diff}");
            let sql = format!(
                "UPDATE difficulties SET diff={} WHERE beatmap_hash='{}' AND playmode='{}' AND mods_string='{}'", 
                diff,
                map_hash,
                playmode,
                mods,
            );
            let mut s = db.prepare(&sql).unwrap();

            if let Err(e) = s.execute([]) {
                println!("[Database] Error inserting/updateing difficulties table: {e}")
            }
        }
    }

    pub fn get_diff(map_hash: &String, playmode: &PlayMode, mods: &ModManager) -> Option<f32> {
        let db = Self::get();
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