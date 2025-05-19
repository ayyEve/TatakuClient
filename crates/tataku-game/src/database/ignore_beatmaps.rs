use crate::prelude::*;

impl Database {
    pub fn get_all_ignored() -> Vec<String> {
        let db = Self::get();
        let mut s = db.prepare("SELECT * FROM ignore_maps").unwrap();
        
        s.query_map([], |r|r.get("beatmap_path")).unwrap()
        .filter_map(|m| {
            if let Err(e) = &m { error!("DB Err: {}", e) }
            m.ok()
        })
        .collect::<Vec<String>>()
    }

    pub fn add_ignored(path: String) {
        let query = "INSERT INTO ignore_maps (beatmap_path, beatmap_hash) VALUES (?, '')";

        let db = Self::get();
        let res = db.prepare(query)
            .expect(query)
            .execute([path]);

        if let Err(e) = res {
            error!("error inserting metadata: {}", e);
        }
    }
}
