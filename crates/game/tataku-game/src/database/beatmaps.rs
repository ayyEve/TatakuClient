use crate::prelude::*;

const BEATMAP_INSERT: &str = "
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
const COLUMNS: usize = 20;

impl Database {
    pub fn get_all_beatmaps() -> Vec<Arc<BeatmapMeta>> {
        let db = Self::get();
        let mut s = db.prepare("SELECT * FROM beatmaps").unwrap();
        
        s.query_map([], row_into_metadata)
            .unwrap()
            .filter_map(|m|{
                if let Err(e) = &m {error!("DB Err: {}", e)}
                m.map(Arc::new).ok()
            })
            .collect::<Vec<Arc<BeatmapMeta>>>()
    }

    pub fn clear_all_maps() {
        let db = Self::get();
        let statement = "DELETE FROM beatmaps";
        let res = db
            .prepare(statement)
            .expect(statement)
            .execute([]);

        if let Err(e) = res {
            error!("error deleting beatmap meta from db: {}", e);
        }
    }

    pub fn insert_beatmaps(maps: &[Arc<BeatmapMeta>]) {
        let max_inserts_per_statement = 1_000;
        let maps_iter = maps
            .chunks(max_inserts_per_statement);

        for map_group in maps_iter {
            let values = map_group
                .iter()
                .flat_map(|a| beatmap_values(a))
                .collect::<Vec<_>>();
            let values = values.iter()
                .map(|i| &**i)
                .collect::<Vec<_>>();

            let query = BEATMAP_INSERT.to_string() + &Self::make_values_str(
                COLUMNS,
                map_group.len()
            );

            let db = Self::get();
            let res = db
                .prepare(&query)
                .expect(&query)
                .execute(&*values);

            if let Err(e) = res {
                error!("Error inserting metadatas: {e}");
            }
        }
    }
}

fn row_into_metadata(r: &rusqlite::Row) -> rusqlite::Result<BeatmapMeta> {
    Ok(BeatmapMeta {
        file_path: r.get("beatmap_path")?,
        beatmap_hash: r.get::<&str, String>("beatmap_hash")?.try_into().unwrap(),
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
    })
}

fn beatmap_values(map: &BeatmapMeta) -> Vec<SqlValue<'_>> {
    let mut bpm_min = Box::new(map.bpm_min);
    let mut bpm_max = Box::new(map.bpm_max);
    if !bpm_min.is_normal() {
        *bpm_min = 0.0;
    }
    if !bpm_max.is_normal() {
        *bpm_max = 99999999.0;
    }
    let beatmap_type:Box<u8> = Box::new(map.beatmap_type.into());
    let hash = Box::new(map.beatmap_hash.to_string());
    
    vec![
        (&map.file_path).into(), hash.into(), beatmap_type.into(),

        (&map.mode).into(),
        (&map.artist).into(), (&map.artist_unicode).into(),
        (&map.title).into(), (&map.title_unicode).into(),
        (&map.creator).into(), (&map.version).into(),
        
        (&map.audio_filename).into(), (&map.image_filename).into(),
        (&map.audio_preview).into(), (&map.duration).into(),

        (&map.hp).into(), (&map.od).into(), (&map.cs).into(), (&map.ar).into(),

        bpm_min.into(), bpm_max.into()
    ]
}
