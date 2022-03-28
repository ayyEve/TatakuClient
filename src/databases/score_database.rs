use crate::prelude::*;
use crate::REPLAYS_DIR;

impl Database {
    pub fn get_scores(hash:&String, playmode:PlayMode) -> Vec<Score> {
        let db = Self::get();
        let mut s = db.prepare(&format!("SELECT * FROM scores WHERE map_hash='{}' AND playmode='{}'", hash, playmode)).unwrap();

        s.query_map([], |r| {
            let _score_hash:String = r.get("score_hash")?;

            let score = Score {
                version: r.get("version").unwrap_or(1), // v1 didnt include version in the table
                username: r.get("username")?,
                playmode: r.get("playmode")?,
                score: r.get("score")?,
                combo: r.get("combo")?,
                max_combo: r.get("max_combo")?,
                x50: r.get("x50").unwrap_or(0),
                x100: r.get("x100")?,
                x300: r.get("x300")?,
                xmiss: r.get("xmiss")?,
                xgeki: r.get("xgeki").unwrap_or_default(),
                xkatu: r.get("xkatu").unwrap_or_default(),
                accuracy: r.get("accuracy").unwrap_or_default(),
                beatmap_hash: r.get("map_hash")?,
                speed: r.get("speed").unwrap_or(1.0),
                hit_timings: Vec::new(),
                replay_string: None,
                mods_string: r.get("mods_string").ok()
            };

            Ok(score)
        })
            .unwrap()
            // .filter_map(|m|m.ok())
            .filter_map(|m| {
                if let Err(e) = &m {
                    println!("score error: {}", e);
                }
                m.ok()
            })
            .collect::<Vec<Score>>()
    }


    pub fn save_score(s:&Score) {
        println!("saving score");
        let db = Self::get();
        let sql = format!(
            "INSERT INTO scores (
                map_hash, score_hash,
                username, playmode,
                score,
                combo, max_combo,
                x50, x100, x300, geki, katu, xmiss,
                speed, 
                version,
                mods_string
            ) VALUES (
                '{}', '{}',
                '{}', '{}',
                {},
                {}, {},
                {}, {}, {}, {}, {}, {},
                {},
                {},
                '{}'
            )", 
            s.beatmap_hash, s.hash(),
            s.username, s.playmode,
            s.score,
            s.combo, s.max_combo,
            s.x50, s.x100, s.x300, s.xgeki, s.xkatu, s.xmiss, 
            s.speed,
            s.version,
            s.mods_string.clone().unwrap_or_default()
        );
        let mut s = db.prepare(&sql).unwrap();
        s.execute([]).unwrap();
    }

}


pub fn save_replay(r:&Replay, s:&Score) -> TatakuResult<()> {
    let mut writer = SerializationWriter::new();
    writer.write(r.clone());

    let hash = s.hash();
    let actual_hash = format!("{:x}", md5::compute(hash));
    let filename = format!("{}/{}.ttkr", REPLAYS_DIR, actual_hash);
    Ok(save_database(&filename, writer)?)
}

pub fn get_local_replay(score_hash:String) -> TatakuResult<Replay> {
    let actual_hash = format!("{:x}", md5::compute(score_hash));
    let fullpath = format!("{}/{}.ttkr", REPLAYS_DIR, actual_hash);

    println!("[Replay] loading replay: {}", fullpath);
    let mut reader = open_database(&fullpath)?;
    Ok(reader.read()?)
}
