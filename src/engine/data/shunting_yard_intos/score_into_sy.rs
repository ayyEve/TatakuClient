use crate::prelude::*;

impl From<&Score> for CustomElementValue {
    fn from(score: &Score) -> Self {
        let mut map = CustomElementMapHelper::default();
        
        map.set("score", score.score);
        map.set("score_fmt", format_number(score.score));

        map.set("combo", score.combo as u32);
        map.set("combo_fmt", format_number(score.combo));
        
        map.set("max_combo", score.max_combo as u32);
        map.set("max_combo_fmt", format_number(score.max_combo));

        map.set("accuracy", score.accuracy as f32);
        map.set("accuracy_fmt", format!("{:.2}", score.accuracy * 100.0));

        map.set("performance", score.performance);
        map.set("performance_fmt", format!("{:.2}", score.performance));
        
        map.set("username", score.username.clone());
        map.set("mods_short", ModManager::short_mods_string(score.mods(), false, &score.playmode));

        // let now = chrono::Utc::now().timestamp() as u64;
        // let time_diff = now as i64 - self.score.time as i64;
        // let time_diff_str = if time_diff < 60 * 5 {
        //     format!(" | {time_diff}s")
        // } else {
        //     String::new()
        // };

        // map.set("time", );


        map.finish()
    }
}