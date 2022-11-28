use crate::prelude::*;
use super::*;

pub struct OsuGameInfo;
#[async_trait]
impl GameModeInfo for OsuGameInfo {
    fn new() -> Self { Self }
    fn display_name(&self) -> &str { "Osu" }

    fn calc_acc(&self, score: &Score) -> f64 {
        let x50  = score.judgments.get("x50").copy_or_default()  as f64;
        let x100 = score.judgments.get("x100").copy_or_default() as f64;
        let x300 = score.judgments.get("x300").copy_or_default() as f64;
        let geki = score.judgments.get("geki").copy_or_default() as f64;
        let katu = score.judgments.get("katu").copy_or_default() as f64;
        let miss = score.judgments.get("xmiss").copy_or_default() as f64;
    
        (50.0 * x50 + 100.0 * (x100 + katu) + 300.0 * (x300 + geki)) 
        / (300.0 * (miss + x50 + x100 + x300 + katu + geki))
    }

    fn get_mods(&self) -> Vec<GameplayModGroup> { 
        vec![
            GameplayModGroup::new("Difficulty")
                .with_mod(HardRock)
                .with_mod(Easy)
            ,
        ]
    }


    fn get_diff_string(&self, info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
        let speed = mods.get_speed();
        let symb = if speed > 1.0 {"+"} else if speed < 1.0 {"-"} else {""};

        let mut secs = format!("{}", info.secs(speed));
        if secs.len() == 1 {secs = format!("0{}", secs)}

        let mut txt = format!(
            "OD: {:.2}{symb} CS: {:.2}{symb} AR: {:.2}{symb} HP: {:.2}{symb}, Len: {}:{}", 
            super::osu::get_od(info, mods),
            super::osu::get_cs(info, mods),
            super::osu::get_ar(info, mods),
            info.get_hp(mods),
            info.mins(speed), secs
        );

        // make sure at least one has a value
        if info.bpm_min != 0.0 || info.bpm_max != 0.0 {
            // one bpm
            if info.bpm_min == info.bpm_max {
                txt += &format!(" BPM: {:.2}", info.bpm_min * speed);
            } else { // multi bpm
                // i think i had it backwards when setting, just make sure its the right way :/
                let min = info.bpm_min.min(info.bpm_max);
                let max = info.bpm_max.max(info.bpm_min);
                txt += &format!(" BPM: {:.2}-{:.2}", min * speed, max * speed);
            }
        }

        if let Some(diff) = &info.diff {
            txt += &format!(", Diff: {:.2}", diff);
        } else {
            txt += &format!(", Diff: ...");
        }

        txt
    }

    fn get_judgments(&self) -> Box<dyn crate::prelude::HitJudgments> {
        Box::new(super::OsuHitJudgments::Miss)
    }
    async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>> {
        let game = super::osu::StandardGame::new(beatmap, false).await?;
        Ok(Box::new(game))
    }
    async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>> {
        let calc = super::diff_calc::OsuDifficultyCalculator::new(map).await?;
        Ok(Box::new(calc))
    }

}