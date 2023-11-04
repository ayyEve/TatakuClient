use crate::prelude::*;
use crate::REPLAYS_DIR;

impl Database {
    pub async fn get_scores(hash:&String, playmode:String) -> Vec<Score> {
        let db = Self::get().await;
        let mut s = db.prepare(&format!("SELECT * FROM scores WHERE map_hash='{}' AND playmode='{}'", hash, playmode)).unwrap();

        s.query_map([], |r| {
            let _score_hash:String = r.get("score_hash")?;

            let mut mods_string:Option<String> = r.get("mods_string").ok();
            if let Some(str) = &mods_string {
                if str.is_empty() {
                    mods_string = None;
                }
            }

            let mut judgments = HashMap::new();

            // string will be key:val|key:val
            let judgment_str = r
                .get::<&str, String>("judgments")
                .ok()
                .and_then(|s| if s.is_empty() {None} else {Some(s)});
            if let Some(judgment_string) = judgment_str {
                judgments = Score::judgments_from_string(&judgment_string);
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

            let mut score = Score::default();
            score.version = r.get("version").unwrap_or(1); // v1 didnt include version in the table
            score.username = r.get("username")?;
            score.playmode = r.get("playmode")?;
            score.time = r.get("time").unwrap_or(0);
            score.score = r.get("score")?;
            score.combo = r.get("combo")?;
            score.max_combo = r.get("max_combo")?;
            score.accuracy = r.get("accuracy").unwrap_or_default();
            score.beatmap_hash = r.get::<&str, String>("map_hash")?.try_into().unwrap();
            score.speed = r.get::<&str, f32>("speed").map(|s|GameSpeed::from_f32(s)).unwrap_or_default();
            score.hit_timings = Vec::new();
            score.judgments = judgments;
            
            if let Some(mods_string) = mods_string {
                // old mods format, json
                if mods_string.contains("{") {
                    *score.mods_mut() = Score::mods_from_old_string(mods_string);
                } else {
                    *score.mods_mut() = Score::mods_from_string(mods_string);
                }
            }


            Ok(score)
        })
            .unwrap()
            // .filter_map(|m|m.ok())
            .filter_map(|m| {
                if let Err(e) = &m {
                    error!("score error: {}", e);
                }
                m.ok()
            })
            .collect::<Vec<Score>>()
    }


    pub async fn save_score(s:&Score) {
        trace!("saving score");

        let db = Self::get().await;
        let sql = format!(
            "INSERT INTO scores (
                map_hash, score_hash,
                username, playmode, time,
                score,
                combo, max_combo,
                x50, x100, x300, geki, katu, xmiss,
                speed, 
                version,
                mods_string,
                judgments
            ) VALUES (
                '{}', '{}',
                '{}', '{}', {},
                {},
                {}, {},
                0, 0, 0, 0, 0, 0,
                {},
                {},
                '{}',
                '{}'
            )", 
            s.beatmap_hash, s.hash(),
            s.username, s.playmode, s.time,
            s.score,
            s.combo, s.max_combo,
            // s.x50, s.x100, s.x300, s.xgeki, s.xkatu, s.xmiss, 
            s.speed,
            s.version,
            s.mods_string_sorted(),
            s.judgment_string()
        );

        match db.prepare(&sql) {
            Ok(mut s) => {
                if let Err(e) = s.execute([]) {
                    error!("error executing query: {e}\n {sql}")
                }
            }

            Err(e) => error!("error preparing query: {e}\n {sql}")
        };
        
    }

}

/// returns the path of the replay
pub fn save_replay(r:&Replay, s:&Score) -> TatakuResult<String> {
    // make sure the replay has score data set
    let mut r = r.clone();
    if r.score_data.is_none() {
        r.score_data = Some(s.clone());
    }

    let mut writer = SerializationWriter::new();
    writer.write(&r);

    let hash = s.hash();
    let actual_hash = format!("{:x}", md5::compute(hash));
    let filename = format!("{}/{}.ttkr", REPLAYS_DIR, actual_hash);
    // info!("Saving replay as {}, judgments: {}", filename, s.judgment_string());
    save_database(&filename, writer)?;
    Ok(filename)
}

pub fn get_local_replay(score_hash:String) -> TatakuResult<Replay> {
    let actual_hash = format!("{:x}", md5::compute(score_hash));
    let fullpath = format!("{}/{}.ttkr", REPLAYS_DIR, actual_hash);
    // info!("loading replay: {fullpath}");
    
    let mut reader = open_database(&fullpath)?;
    Ok(reader.read()?)
}

pub fn get_local_replay_for_score(score: &Score) -> TatakuResult<Replay> {
    // info!("Loading replay, judgments: {:#?}", score.hash());
    get_local_replay(score.hash())
}