use crate::prelude::*;
use super::super::prelude::*;

pub struct UTypingGameInfo;
#[async_trait]
impl GameModeInfo for UTypingGameInfo {
    fn new() -> Self { Self }
    fn display_name(&self) -> &'static str { "UTyping" }

    fn calc_acc(&self, score: &Score) -> f32 {
        let x100 = score.judgments.get("x100").copy_or_default() as f32;
        let x300 = score.judgments.get("x300").copy_or_default() as f32;
        let miss = score.judgments.get("xmiss").copy_or_default() as f32;

        (x100 / 2.0 + x300) 
        / (miss + x100 + x300)
    }

    fn get_mods(&self) -> Vec<GameplayModGroup> { 
        vec![
            // GameplayModGroup::new("Skill")
            //     .with_mod(super::FullAlt)
            //     .with_mod(super::Relax)
            //     .with_mod(super::NoFinisher)
            // ,
            // GameplayModGroup::new("Difficulty")
            //     .with_mod(super::HardRock)
            //     .with_mod(super::Easy)
            //     .with_mod(super::NoBattery)
            // ,
        ]
    }

    fn get_stat_groups(&self) -> Vec<StatGroup> {
        vec![
            // StatGroup::new("press_counters", "Press Counts")
            //     .with_stat(TaikoStatLeftPresses)
            //     .with_stat(TaikoStatRightPresses)
        ]
    }

    fn get_diff_string(&self, info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
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

    fn get_judgments(&self) -> Vec<HitJudgment> {
        UTypingHitJudgment::variants().to_vec()
    }
    async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>> {
        let game = UTypingGame::new(beatmap, false).await?;
        Ok(Box::new(game))
    }
    async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>> {
        let calc = UTypingDifficultyCalculator::new(map).await?;
        Ok(Box::new(calc))
    }
}