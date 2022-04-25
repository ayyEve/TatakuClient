use crate::prelude::*;

impl Database {
    pub async fn get_all_beatmaps() -> Vec<BeatmapMeta> {
        let db = Self::get().await;
        let mut s = db.prepare("SELECT * FROM beatmaps").unwrap();
        
        s.query_map([], row_into_metadata).unwrap()
            .filter_map(|m|{
                if let Err(e) = &m {error!("DB Err: {}", e)}
                m.ok()
            })
            .collect::<Vec<BeatmapMeta>>()
    }

    pub async fn clear_all_maps() {
        let db = Self::get().await;
        let statement = format!("DELETE FROM beatmaps");
        let res = db.prepare(&statement).expect(&statement).execute([]);
        if let Err(e) = res {
            error!("error deleting beatmap meta from db: {}", e);
        }
    }

    pub async fn insert_beatmap(map: &BeatmapMeta) {
        let mut bpm_min = map.bpm_min;
        let mut bpm_max = map.bpm_max;
        if !bpm_min.is_normal() {
            bpm_min = 0.0;
        }
        if !bpm_max.is_normal() {
            bpm_max = 99999999.0;
        }
        let beatmap_type:u8 = map.beatmap_type.into();
    
        let query = format!(
            "INSERT INTO beatmaps (
                beatmap_path, beatmap_hash, beatmap_type,
    
                playmode, 
                artist, artist_unicode,
                title, title_unicode,
                creator, version,
    
                audio_filename, image_filename,
                audio_preview, duration,
                
                hp, od, cs, ar,
                
                bpm_min, bpm_max
            ) VALUES (
                \"{}\", \"{}\", {},
    
                \"{}\",
                \"{}\", \"{}\",
                \"{}\", \"{}\",
                \"{}\", \"{}\",
    
                \"{}\", \"{}\",
                {}, {},
    
                {}, {}, {}, {},
    
                {}, {}
            )",
            map.file_path, map.beatmap_hash, beatmap_type,
    
            map.mode,
            map.artist.replace("\"", "\"\""), map.artist_unicode.replace("\"", "\"\""),
            map.title.replace("\"", "\"\""), map.title_unicode.replace("\"", "\"\""),
            map.creator.replace("\"", "\"\""), map.version.replace("\"", "\"\""),
            
            map.audio_filename, map.image_filename,
            map.audio_preview, map.duration,
    
            map.hp, map.od, map.cs, map.ar,
    
            bpm_min, bpm_max
        );

        let db = Self::get().await;
        let res = db.prepare(&query).expect(&query).execute([]);
        if let Err(e) = res {
            error!("error inserting metadata: {}", e);
        }
    }
}

fn row_into_metadata(r: &rusqlite::Row) -> rusqlite::Result<BeatmapMeta> {
    Ok(BeatmapMeta {
        file_path: r.get("beatmap_path")?,
        beatmap_hash: r.get("beatmap_hash")?,
        beatmap_type: r.get::<&str, u8>("beatmap_type")?.into(),
        mode: r.get("playmode")?,
        artist: r.get("artist")?,
        title: r.get("title")?,
        artist_unicode: r.get("artist_unicode")?,
        title_unicode: r.get("title_unicode")?,
        creator: r.get("creator")?,
        version: r.get("version")?,
        audio_filename: r.get("audio_filename")?,
        image_filename: r.get("image_filename")?,
        audio_preview: r.get("audio_preview")?,
        hp: r.get("hp")?,
        od: r.get("od")?,
        ar: r.get("ar")?,
        cs: r.get("cs")?,

        duration: r.get("duration")?,
        bpm_min: r.get("bpm_min").unwrap_or(0.0),
        bpm_max: r.get("bpm_max").unwrap_or(0.0),

        diff: -1.0
    })
}