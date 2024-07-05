use crate::prelude::*;
use super::super::prelude::*;

pub struct ManiaGameInfo;
#[async_trait]
impl GameModeInfo for ManiaGameInfo {
    fn new() -> Self { Self }
    fn display_name(&self) -> &'static str { "Mania" }

    /// from https://wiki.quavergame.com/docs/gameplay#accuracy
    fn calc_acc(&self, score: &Score) -> f64 {
        let marv = score.judgments.get("geki").copy_or_default() as f64;
        let perf = score.judgments.get("x300").copy_or_default() as f64;
        let great = score.judgments.get("katu").copy_or_default() as f64;
        let good = score.judgments.get("x100").copy_or_default() as f64;
        let okay  = score.judgments.get("x50").copy_or_default() as f64;
        let miss = score.judgments.get("xmiss").copy_or_default() as f64;
    
        let top:f64 = [
            marv * 1.0, // 100%
            perf * 0.9825, // 98.25%
            great * 0.65, // 65%
            good * 0.25, // 25%
            okay * -1.00, // -100%
            miss * -0.50, // -50%
        ].iter().sum();
    
        let bottom:f64 = [
            marv, 
            perf, 
            great, 
            good, 
            okay, 
            miss
        ].iter().sum();
    
        top.max(0.0) / bottom
    }

    fn get_mods(&self) -> Vec<GameplayModGroup> { 
        vec![]
    }


    fn get_perf_calc(&self) -> PerformanceCalc where Self:Sized {
        Box::new(|diff, acc| diff * (acc / 0.98).powi(6))
    }

    fn get_diff_string(&self, info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
        let speed = mods.get_speed();
        // let symb = if speed > 1.0 {"+"} else if speed < 1.0 {"-"} else {""};

        let mut secs = format!("{}", info.secs(speed));
        if secs.len() == 1 {secs = format!("0{}", secs)}

        let mut txt = format!("Keys: {:0} Len: {}:{}", info.cs, info.mins(speed), secs);

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

    fn get_judgments(&self) -> Vec<HitJudgment> {
        ManiaHitJudgments::variants().to_vec()
    }
    async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>> {
        let game = ManiaGame::new(beatmap, false).await?;
        Ok(Box::new(game))
    }
    async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>> {
        let calc = ManiaDifficultyCalculator::new(map).await?;
        Ok(Box::new(calc))
    }

}