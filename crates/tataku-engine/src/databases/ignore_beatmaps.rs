use crate::prelude::*;

impl Database {
    pub async fn get_all_ignored() -> Vec<String> {
        let db = Self::get().await;
        let mut s = db.prepare("SELECT * FROM ignore_maps").unwrap();
        
        s.query_map([], |r|r.get("beatmap_path")).unwrap()
        .filter_map(|m| {
            if let Err(e) = &m { error!("DB Err: {}", e) }
            m.ok()
        })
        .collect::<Vec<String>>()
    }

    // pub async fn clear_all_ignored() {
    //     let db = Self::get().await;
    //     let statement = format!("DELETE FROM ignore_maps");
    //     let res = db.prepare(&statement).expect(&statement).execute([]);
    //     if let Err(e) = res {
    //         error!("error deleting beatmap meta from db: {}", e);
    //     }
    // }

    pub async fn add_ignored(path: String) {
        let query = "INSERT INTO ignore_maps (beatmap_path, beatmap_hash) VALUES (?, '')";

        let db = Self::get().await;
        let res = db.prepare(query).expect(query).execute([path]);
        if let Err(e) = res {
            error!("error inserting metadata: {}", e);
        }
    }
}
