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


// TODO: please god we need a proc macro
impl TryInto<Score> for &CustomElementValue {
    type Error = String;

    fn try_into(self) -> Result<Score, Self::Error> {
        let CustomElementValue::Map(map) = self else { return Err(format!("Not a map")) };
        
        let mut score = Score::new(
            map.get("beatmap_hash").ok_or_else(|| format!("no beatmap_hash?"))?.try_into().map_err(|e| format!("beatmap_hash read error: {e}"))?,
            map.get("username").ok_or_else(|| format!("no username?"))?.as_string(),
            map.get("playmode").ok_or_else(|| format!("no playmode?"))?.as_string(),
        );
        score.time = map.get("time").ok_or_else(|| format!("no time"))?.as_u64().map_err(|e| format!("{e:?}"))?;
        
        score.score = map.get("score").ok_or_else(|| format!("no score"))?.as_u64().map_err(|e| format!("{e:?}"))?;
        score.combo = map.get("combo").ok_or_else(|| format!("no combo"))?.as_u32().map_err(|e| format!("{e:?}"))? as u16;
        score.max_combo = map.get("max_combo").ok_or_else(|| format!("no max_combo"))?.as_u32().map_err(|e| format!("{e:?}"))? as u16;
        
        {
            let judgments = map.get("judgments").ok_or_else(|| format!("no combo"))?.as_map().ok_or_else(|| format!("judgments not a map"))?;
            for (j, c) in judgments {
                let Ok(c) = c.as_u32() else { continue };
                score.judgments.insert(j.clone(), c as u16);
            }
        }
        score.accuracy = map.get("accuracy").ok_or_else(|| format!("no accuracy"))?.as_f32().map_err(|e| format!("{e:?}"))? as f64;
        score.speed = GameSpeed::from_u16(map.get("speed").ok_or_else(|| format!("no speed"))?.as_u32().map_err(|e| format!("{e:?}"))? as u16);
        score.performance = map.get("performance").ok_or_else(|| format!("no performance"))?.as_f32().map_err(|e| format!("{e:?}"))?;
        
        let mods = ModManager::try_from(
            map.get("mods").ok_or_else(|| format!("no mods"))?
        ).map_err(|_| format!("bad mods"))?;
        mods.mods.into_iter().for_each(|m|score.add_mod(m));

        if let CustomElementValue::List(list) = map.get("hit_timings").ok_or_else(|| format!("no hit_timings"))? {
            for i in list {
                score.hit_timings.push(i.as_f32().map_err(|e| format!("{e:?}"))?);
            }
        }

        let stats = map.get("stats").ok_or_else(|| format!("no stats"))?.as_map().ok_or_else(|| format!("no hit_timings"))?;
        for (stat, thing) in stats {
            let mut val_list = Vec::new();
            if let CustomElementValue::List(list) = thing {
                for i in list {
                    let v = i.as_f32().map_err(|e| format!("{e:?}"))?;
                    val_list.push(v);
                }
            }
            score.stat_data.insert(stat.clone(), val_list);
        }


        Ok(score)
    }
}