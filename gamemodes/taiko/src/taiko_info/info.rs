use crate::prelude::*;

pub static GAME_INFO: GamemodeInfo = GamemodeInfo {
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
    diff_values: &[
        OVERALL_DIFFICULTY,

        BPM_DIFF_VALUE,
        DURATION_DIFF_VALUE,
    ],

    stat_groups: &[
        TaikoPressCounterStatGroup
    ],
    available_widgets: &[ DON_CHAN ],

    judgments: super::TaikoHitJudgments::variants(),

    calc_acc: TaikoGameInfo::calc_acc,
    // get_diff_string: TaikoGameInfo::get_diff_string,
    stats_from_groups: TaikoGameInfo::stats_from_groups,
    create_game: TaikoGameInfo::create_game,
    create_diffcalc: TaikoGameInfo::create_diffcalc,
    can_load_beatmap: |map| matches!(map, BeatmapType::Osu | BeatmapType::Tja),

    serialize_settings: TaikoGameInfo::serialize_settings,
    deserialize_settings: TaikoGameInfo::deserialize_settings,

    ..GamemodeInfo::DEFAULT
};


struct TaikoGameInfo;
impl TaikoGameInfo {
    fn calc_acc(score: &Score) -> f32 {
        let x100 = score.judgments.get("x100").copied().unwrap_or_default() as f32;
        let x300 = score.judgments.get("x300").copied().unwrap_or_default() as f32;
        let miss = score.judgments.get("xmiss").copied().unwrap_or_default() as f32;

        (x100 / 2.0 + x300) 
        / (miss + x100 + x300)
    }

    #[cfg(feature="graphics")]
    fn stats_from_groups(data: &HashMap<String, HashMap<String, Vec<f32>>>) -> Vec<StatsInfo> { 
        let mut info = Vec::new();

        macro_rules! get_or_return {
            ($data: expr, $thing:expr) => {
                if let Some(val) = $data.get(&$thing.name().to_owned()) { val } else { return info }
            }
        }

        if let Some(press_counters) = data.get(&"press_counters".to_owned()) {
            let left_presses:f32 = get_or_return!(press_counters, TaikoStatLeftPresses).iter().sum();
            let right_presses:f32 = get_or_return!(press_counters, TaikoStatRightPresses).iter().sum();
            info.push(StatsInfo::new("Presses", GraphType::Pie, vec![
                StatsEntry::new_f32("Left Presses", left_presses, Color::BLUE, true, true),
                StatsEntry::new_f32("Right Presses", right_presses, Color::RED, true, true),
            ]))
        }

        info
    }


    fn create_game(beatmap: &Beatmap, settings: &Settings) -> TatakuResult<Box<dyn GameMode>> {
        Ok(Box::new(TaikoGame::new(beatmap, false, settings)?))
    }
    fn create_diffcalc(map: &BeatmapMeta, settings: &Settings) -> TatakuResult<Box<dyn DiffCalc>> {
        Ok(Box::new(TaikoDifficultyCalculator::new(map, settings)?))
    }


    fn deserialize_settings(value: serde_json::Value) -> Option<Box<dyn GamemodeSettings>> {
        if value.is_null() {
            let settings: Box<dyn GamemodeSettings> = Box::new(TaikoSettings::default());
            return Some(settings);
        }

        let parsed = serde_json::from_value::<TaikoSettings>(value).ok()?; 
        let a: Box<dyn GamemodeSettings> = Box::new(parsed);
        Some(a)
    }
    fn serialize_settings(s: Box<dyn GamemodeSettings>) -> serde_json::Value {
        let s = *s.downcast::<TaikoSettings>().unwrap();
        serde_json::to_value(&s).unwrap()
    }
}


pub const OVERALL_DIFFICULTY: DifficultyValue = DifficultyValue {
    id: "od",
    name: "OD",
    modifiable: true,
    number_type: DifficultyNumberType::Float,
    min: 0.0,
    max: 11.0,
    step: Some(0.1),
    unit: None,
    get_diff_value: |map, mods| TaikoGame::get_od(map, mods),
};
