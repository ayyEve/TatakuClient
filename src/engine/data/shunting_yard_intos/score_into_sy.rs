use crate::prelude::*;

impl From<&Score> for TatakuValue {
    fn from(score: &Score) -> Self {
        let mut map = ValueCollectionMapHelper::default();

        map.set("username", TatakuVariable::new(score.username.clone()));
        map.set("beatmap_hash", TatakuVariable::new(score.beatmap_hash));
        map.set("playmode", TatakuVariable::new(&score.playmode));
        map.set("time", TatakuVariable::new(score.time));
        
        map.set("score", TatakuVariable::new(score.score).display(format_number(score.score)));
        map.set("combo", TatakuVariable::new(score.combo as u32).display(format_number(score.combo)));
        map.set("max_combo", TatakuVariable::new(score.max_combo as u32).display(format_number(score.max_combo)));

        // judgments 
        {
            let mut judgments = ValueCollectionMapHelper::default();
            for (judge, count) in &score.judgments {
                judgments.set(judge, TatakuVariable::new(*count as u32))
            }
            map.set("judgments", TatakuVariable::new(judgments.finish()));
        }

        map.set("accuracy", TatakuVariable::new(score.accuracy as f32).display(format!("{:.2}", score.accuracy * 100.0)));
        map.set("speed", TatakuVariable::new(score.speed.as_u16() as u32));

        map.set("performance", TatakuVariable::new(score.performance).display(format!("{:.2}", score.performance)));


        
        // map.set("mods_short", ));

        map.set("mods", TatakuVariable::new(ModManager::new().with_mods(score.mods.iter()).with_speed(score.speed)).display(ModManager::short_mods_string(&score.mods, false, &score.playmode)));
        map.set("hit_timings", TatakuVariable::new((TatakuVariableAccess::ReadOnly, score.hit_timings.clone())));

        

        // stats
        {
            let mut stats = ValueCollectionMapHelper::default();
            for (stat, list) in &score.stat_data {
                stats.set(stat, TatakuVariable::new((TatakuVariableAccess::ReadOnly, list.clone())));
            }
            map.set("stats", TatakuVariable::new(stats.finish()));
        }

        map.finish()
    }
}


// TODO: please god we need a proc macro
impl TryInto<Score> for &TatakuValue {
    type Error = String;

    fn try_into(self) -> Result<Score, Self::Error> {
        let TatakuValue::Map(map) = self else { return Err(format!("Not a map")) };
        
        let mut score = Score::new(
            (&map.get("beatmap_hash").ok_or_else(|| format!("no beatmap_hash?"))?.value).try_into().map_err(|e| format!("beatmap_hash read error: {e}"))?,
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
        score.accuracy = map.get("accuracy").ok_or_else(|| format!("no accuracy"))?.as_f32().map_err(|e| format!("{e:?}"))?;
        score.speed = GameSpeed::from_u16(map.get("speed").ok_or_else(|| format!("no speed"))?.as_u32().map_err(|e| format!("{e:?}"))? as u16);
        score.performance = map.get("performance").ok_or_else(|| format!("no performance"))?.as_f32().map_err(|e| format!("{e:?}"))?;
        
        let ok_mods = ModManager::mods_for_playmode_as_hashmap(&score.playmode);

        let mods = ModManager::try_from(
            &map.get("mods").ok_or_else(|| format!("no mods"))?.value
        ).map_err(|_| format!("bad mods"))?;
        mods.mods.iter().filter_map(|m| ok_mods.get(m)).for_each(|m| score.mods.push((*m).into()));

        if let TatakuValue::List(list) = &map.get("hit_timings").ok_or_else(|| format!("no hit_timings"))?.value {
            for i in list {
                score.hit_timings.push(i.as_f32().map_err(|e| format!("{e:?}"))?);
            }
        }

        let stats = map.get("stats").ok_or_else(|| format!("no stats"))?.as_map().ok_or_else(|| format!("no hit_timings"))?;
        for (stat, thing) in stats {
            let mut val_list = Vec::new();
            if let TatakuValue::List(list) = &thing.value {
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