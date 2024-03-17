use crate::prelude::*;

impl From<&Score> for CustomElementValue {
    fn from(score: &Score) -> Self {
        let mut map = CustomElementMapHelper::default();

        map.set("username", score.username.clone());
        map.set("beatmap_hash", score.beatmap_hash);
        map.set("playmode", &score.playmode);
        map.set("time", score.time);
        
        map.set("score", score.score);
        map.set("score_fmt", format_number(score.score));

        map.set("combo", score.combo as u32);
        map.set("combo_fmt", format_number(score.combo));
        
        map.set("max_combo", score.max_combo as u32);
        map.set("max_combo_fmt", format_number(score.max_combo));

        // judgments 
        {
            let mut judgments = CustomElementMapHelper::default();
            for (judge, count) in &score.judgments {
                judgments.set(judge, *count as u32)
            }
            map.set("judgments", judgments.finish());
        }

        map.set("accuracy", score.accuracy as f32);
        map.set("accuracy_fmt", format!("{:.2}", score.accuracy * 100.0));

        map.set("speed", score.speed.as_u16() as u32);

        map.set("performance", score.performance);
        map.set("performance_fmt", format!("{:.2}", score.performance));


        
        map.set("mods_short", ModManager::short_mods_string(score.mods(), false, &score.playmode));

        map.set("mods", ModManager::new().with_mods(score.mods()).with_speed(score.speed));

        map.set("hit_timings", score.hit_timings.clone());

        

        // stats
        {
            let mut stats = CustomElementMapHelper::default();
            for (stat, list) in &score.stat_data {
                stats.set(stat, list.clone());
            }
            map.set("stats", stats.finish());
        }

        map.finish()
    }
}


impl TryInto<Score> for &CustomElementValue {
    type Error = String;

    fn try_into(self) -> Result<Score, Self::Error> {
        let CustomElementValue::Map(map) = self else { return Err(format!("Not a map")) };
        
        let mut score = Score::new(
            map.get("beatmap_hash").ok_or_else(|| format!("no beatmap_hash?"))?.try_into().map_err(|e| format!("beatmap_hash read error: {e}"))?,
            map.get("username").ok_or_else(|| format!("no username?"))?.as_string(),
            map.get("playmode").ok_or_else(|| format!("no playmode?"))?.as_string(),
        );



        Ok(score)
    }
}