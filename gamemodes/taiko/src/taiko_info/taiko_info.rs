use crate::prelude::*;
use futures_util::future::BoxFuture;

pub static GAME_INFO: GameModeInfo = GameModeInfo {
    id: "taiko",
    display_name: "Taiko",
    about: "Taiko!",
    author: "ayyEve",

    mods: &[
        GameplayModGroupStatic {
            name: "Skill",
            mods: &[
                FullAlt,
                Relax,
                NoFinisher,
                NoSV,
            ]
        },
        GameplayModGroupStatic {
            name: "Difficulty",
            mods: &[
                HardRock,
                Easy,
                Flashlight,
                NoBattery,
            ]
        },
    ],
    stat_groups: &[
        TaikoPressCounterStatGroup
    ],
    judgments: super::TaikoHitJudgments::variants(),

    calc_acc: TaikoGameInfo::calc_acc,
    get_diff_string: TaikoGameInfo::get_diff_string,
    stats_from_groups: TaikoGameInfo::stats_from_groups,
    create_game: TaikoGameInfo::create_game,
    create_diffcalc: TaikoGameInfo::create_diffcalc,
    can_load_beatmap: TaikoGameInfo::can_load_beatmap,

    ..GameModeInfo::DEFAULT
};



#[derive(Debug)]
struct TaikoGameInfo;
impl TaikoGameInfo {
    fn calc_acc(score: &Score) -> f32 {
        let x100 = score.judgments.get("x100").copied().unwrap_or_default() as f32;
        let x300 = score.judgments.get("x300").copied().unwrap_or_default() as f32;
        let miss = score.judgments.get("xmiss").copied().unwrap_or_default() as f32;

        (x100 / 2.0 + x300) 
        / (miss + x100 + x300)
    }

    fn can_load_beatmap(map: &BeatmapType) -> bool { 
        match map {
            BeatmapType::Osu => true,
            BeatmapType::Tja => true,
            _ => false
        }
    }

    fn get_diff_string(info: &BeatmapMetaWithDiff, mods: &ModManager) -> String {
        let speed = mods.get_speed();
        let symb = if speed > 1.0 {"+"} else if speed < 1.0 {"-"} else {""};

        let mut secs = format!("{}", info.secs(speed));
        if secs.len() == 1 {secs = format!("0{}", secs)}

        let mut txt = format!(
            "OD: {:.2}{symb} HP: {:.2}{symb}, Len: {}:{}", 
            TaikoGame::get_od(info, mods),
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

    #[cfg(feature="graphics")]
    fn stats_from_groups(data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<MenuStatsInfo> { 
        let mut info = Vec::new();

        macro_rules! get_or_return {
            ($data: expr, $thing:expr) => {
                if let Some(val) = $data.get(&$thing.name().to_owned()) { val } else { return info }
            }
        }

        if let Some(press_counters) = data.get(&"press_counters".to_owned()) {
            let left_presses:f32 = get_or_return!(press_counters, TaikoStatLeftPresses).iter().sum();
            let right_presses:f32 = get_or_return!(press_counters, TaikoStatRightPresses).iter().sum();
            info.push(MenuStatsInfo::new("Presses", GraphType::Pie, vec![
                MenuStatsEntry::new_f32("Left Presses", left_presses, Color::BLUE, true, true),
                MenuStatsEntry::new_f32("Right Presses", right_presses, Color::RED, true, true),
            ]))
        }

        info
    }


    fn create_game<'a>(beatmap: &'a Beatmap, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn GameMode>>> {
        Box::pin(async {
            let game:Box<dyn GameMode> = Box::new(TaikoGame::new(beatmap, false, settings).await?);
            Ok(game)
        })
    }
    fn create_diffcalc<'a>(map: &'a BeatmapMeta, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn DiffCalc>>> {
        Box::pin(async {
            let calc:Box<dyn DiffCalc> = Box::new(TaikoDifficultyCalculator::new(map, settings).await?);
            Ok(calc)
        })
    }



}