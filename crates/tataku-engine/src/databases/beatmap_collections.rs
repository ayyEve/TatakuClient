#![allow(unused)]
use crate::prelude::*;


pub struct BeatmapCollection {
    pub collection_name: String,
    pub beatmaps: Vec<String>
}

struct BeatmapCollectionEntry {
    collection_name: String,
    beatmap: String
}
impl BeatmapCollectionEntry {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            collection_name: row.get("collection_name")?,
            beatmap: row.get("beatmap_hash")?,
        })
    }
}


impl Database {
    pub fn init_beatmap_collection(connection: &rusqlite::Connection) {

        // difficulties table
        connection.execute(
            "CREATE TABLE IF NOT EXISTS beatmap_collections (
                beatmap_hash TEXT,
                collection_name TEXT,

                PRIMARY KEY (beatmap_hash, collection_name)
            )", [])
        .expect("error creating db table");
    }


    pub async fn get_beatmap_collections() -> Vec<BeatmapCollection> {
        let db = Self::get().await;

        let query = "SELECT * FROM beatmap_collections".to_string();
        let mut s = db.prepare(&query).unwrap();
        let res = s.query_map([], BeatmapCollectionEntry::from_row);

        if let Ok(rows) = res {
            let mut map = HashMap::new();
            for i in rows.filter_map(|r|r.ok()) {
                if !map.contains_key(&i.collection_name) {
                    map.insert(i.collection_name.clone(), Vec::new());
                }
                map.get_mut(&i.collection_name).unwrap().push(i.beatmap.clone())
            }

            let mut list = Vec::new();
            for (collection_name, beatmaps) in map {
                list.push(BeatmapCollection {
                    collection_name,
                    beatmaps
                });
            }
            list

        } else {
            Vec::new()
        }
    }

    pub async fn insert_into_beatmap_collection(collection_name: String, beatmap_hash: String) {
        let db = Self::get().await;

        let query = format!("INSERT INTO beatmap_collections (collection_name, beatmap_hash) VALUES ('{collection_name}', '{beatmap_hash}')");
        let mut s = db.prepare(&query).unwrap();

        if let Err(e) = s.execute([]) {
            info!("error adding map into collection: {e}")
        }
    }
}