use crate::prelude::*;
use super::super::prelude::*;

pub struct OsuGameInfo;
#[async_trait]
impl GameModeInfo for OsuGameInfo {
    fn new() -> Self { Self }
    fn display_name(&self) -> &'static str { "Osu" }

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
        // let mut easing_type_group = GameplayModGroup::new("Easing Types");
        // for (name, short_name, display_name, desc, removes) in [
        //     ("in", "IET", "In Easing", "Use in easing method", &["out", "inout", "on_the_beat"] ),
        //     ("out", "OET", "Out Easing", "Use out easing method", &["in", "inout", "on_the_beat"] ),
        //     ("inout", "IOET", "InOut Easing", "Use in-out easing method", &["in", "out", "on_the_beat"] ),
        // ] {
        //     easing_type_group = easing_type_group.with_mod(EasingMod { name, short_name, display_name, desc, removes })
        // }

        let mut easing_group = GameplayModGroup::new("Easing");
        for (name, short_name, display_name, description, removes) in [
            // ("sine", "SE", "Sine Easing", "Approach circles have sine wave easing",         &[        "quad", "cube", "quart", "quint", "exp", "circ", "back", "on_the_beat"] ),
            // ("quad", "2E", "Quadratic Easing", "Approach circles have quadratic easing",    &["sine",         "cube", "quart", "quint", "exp", "circ", "back", "on_the_beat"] ),
            // ("cube", "3E", "Cubic Easing", "Approach circles have cubic easing",            &["sine", "quad",         "quart", "quint", "exp", "circ", "back", "on_the_beat"] ),
            // ("quart", "4E", "Quartic Easing", "Approach circles have quartic easing",       &["sine", "quad", "cube",          "quint", "exp", "circ", "back", "on_the_beat"] ),
            // ("quint", "5E", "Quintic Easing", "Approach circles have quintic easing",       &["sine", "quad", "cube", "quart",          "exp", "circ", "back", "on_the_beat"] ),
            ("exp", "XE", "Exponential Easing", "Approach circles have exponential easing", &["sine", "quad", "cube", "quart", "quint",        "circ", "back", "on_the_beat"] ),
            // ("circ", "CE", "Circular Easing", "Approach circles have circular easing",      &["sine", "quad", "cube", "quart", "quint", "exp",         "back", "on_the_beat"] ),
            // ("back", "BE", "Back Easing", "Approach circles have back easing",              &["sine", "quad", "cube", "quart", "quint", "exp", "circ"        , "on_the_beat"] ),
        ] {
            easing_group = easing_group.with_mod(GameplayMod { name, short_name, display_name, description, removes, ..Default::default() })
        }


        vec![
            GameplayModGroup::new("Difficulty")
                .with_mod(Flashlight)
                .with_mod(HardRock)
                .with_mod(Easy)
                .with_mod(Relax)
            ,
            GameplayModGroup::new("Fun")
                .with_mod(OnTheBeat)
            ,
            // easing_type_group,
            easing_group,
        ]
    }


    fn get_diff_string(&self, info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
        let speed = mods.get_speed();
        let symb = if speed > 1.0 {"+"} else if speed < 1.0 {"-"} else {""};

        let mut secs = format!("{}", info.secs(speed));
        if secs.len() == 1 {secs = format!("0{}", secs)}

        let mut txt = format!(
            "OD: {:.2}{symb} CS: {:.2}{symb} AR: {:.2}{symb} HP: {:.2}{symb}, Len: {}:{}", 
            OsuGame::get_od(info, mods),
            OsuGame::get_cs(info, mods),
            OsuGame::get_ar(info, mods),
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
        Box::new(OsuHitJudgments::Miss)
    }
    async fn create_game(&self, beatmap: &Beatmap) -> TatakuResult<Box<dyn GameMode>> {
        let game = OsuGame::new(beatmap, false).await?;
        Ok(Box::new(game))
    }
    async fn create_diffcalc(&self, map: &BeatmapMeta) -> TatakuResult<Box<dyn DiffCalc>> {
        let calc = OsuDifficultyCalculator::new(map).await?;
        Ok(Box::new(calc))
    }

}