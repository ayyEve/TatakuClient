use crate::prelude::*;
use futures_util::future::BoxFuture;


pub const GAME_INFO: GameModeInfo = GameModeInfo {
    id: "utyping",
    display_name: "uTyping",
    about: "utyping",
    author: "ayyEve",

    mods: &[],

    judgments: UTypingHitJudgment::variants(),
    calc_acc: &UTypingGameInfo::calc_acc,
    get_diff_string: &UTypingGameInfo::get_diff_string,
    create_game: &UTypingGameInfo::create_game,
    create_diffcalc: &UTypingGameInfo::create_diffcalc,

    .. GameModeInfo::DEFAULT
};


struct UTypingGameInfo;
impl UTypingGameInfo {
    fn calc_acc(score: &Score) -> f32 {
        let x100 = score.judgments.get("x100").copied().unwrap_or_default() as f32;
        let x300 = score.judgments.get("x300").copied().unwrap_or_default() as f32;
        let miss = score.judgments.get("xmiss").copied().unwrap_or_default() as f32;

        (x100 / 2.0 + x300) 
        / (miss + x100 + x300)
    }


    fn get_diff_string(info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
        let speed = mods.get_speed();
        let symb = if speed > 1.0 {"+"} else if speed < 1.0 {"-"} else {""};

        let mut secs = format!("{}", info.secs(speed));
        if secs.len() == 1 {secs = format!("0{}", secs)}

        let mut txt = format!(
            "HP: {:.2}{symb}, Len: {}:{}", 
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

    fn create_game(beatmap: &Beatmap) -> BoxFuture<TatakuResult<Box<dyn GameMode>>> {
        Box::pin(async {
            let game:Box<dyn GameMode> = Box::new(UTypingGame::new(beatmap, false).await?);
            Ok(game)
        })
    }
    fn create_diffcalc(map: &BeatmapMeta) -> BoxFuture<TatakuResult<Box<dyn DiffCalc>>> {
        Box::pin(async {
            let calc:Box<dyn DiffCalc> = Box::new(UTypingDifficultyCalculator::new(map).await?);
            Ok(calc)
        })
    }
}