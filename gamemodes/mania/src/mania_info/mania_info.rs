use crate::prelude::*;
use futures_util::future::BoxFuture;

pub const GAME_INFO:GameModeInfo = GameModeInfo {
    id: "mania",
    display_name: "Mania",
    about: "mania!",
    author: "ayyEve",

    mods: &[],

    judgments: ManiaHitJudgments::variants(),
    calc_acc: ManiaGameInfo::calc_acc,
    get_diff_string: ManiaGameInfo::get_diff_string,
    create_game: ManiaGameInfo::create_game,
    create_diffcalc: ManiaGameInfo::create_diffcalc,

    .. GameModeInfo::DEFAULT
};

struct ManiaGameInfo;
impl ManiaGameInfo {
    /// from https://wiki.quavergame.com/docs/gameplay#accuracy
    fn calc_acc(score: &Score) -> f32 {
        let marv = score.judgments.get("geki").copied().unwrap_or_default() as f32;
        let perf = score.judgments.get("x300").copied().unwrap_or_default() as f32;
        let great = score.judgments.get("katu").copied().unwrap_or_default() as f32;
        let good = score.judgments.get("x100").copied().unwrap_or_default() as f32;
        let okay  = score.judgments.get("x50").copied().unwrap_or_default() as f32;
        let miss = score.judgments.get("xmiss").copied().unwrap_or_default() as f32;
    
        let top:f32 = [
            marv * 1.0, // 100%
            perf * 0.9825, // 98.25%
            great * 0.65, // 65%
            good * 0.25, // 25%
            okay * -1.00, // -100%
            miss * -0.50, // -50%
        ].iter().sum();
    
        let bottom:f32 = [
            marv, 
            perf, 
            great, 
            good, 
            okay, 
            miss
        ].iter().sum();
    
        top.max(0.0) / bottom
    }

    fn get_diff_string(info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
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

    fn create_game<'a>(beatmap: &'a Beatmap, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn GameMode>>> {
        Box::pin(async {
            let game: Box<dyn GameMode> = Box::new(ManiaGame::new(beatmap, false, settings).await?);
            Ok(game)
        })
    }
    fn create_diffcalc<'a>(map: &'a BeatmapMeta, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn DiffCalc>>> {
        Box::pin(async {
            let calc:Box<dyn DiffCalc> = Box::new(ManiaDifficultyCalculator::new(map, settings).await?);
            Ok(calc)
        })
    }

}