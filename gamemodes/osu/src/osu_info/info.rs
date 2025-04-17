use crate::prelude::*;

pub const GAME_INFO: GamemodeInfo = GamemodeInfo {
    id: "osu",
    display_name: "Osu",
    about: "osu!",
    author: "ayyEve",

    mods: &[
        GameplayModGroupStatic {
            name: "Difficulty",
            mods: &[
                Flashlight,
                HardRock,
                Easy,
                Relax,
            ]
        },
        GameplayModGroupStatic {
            name: "Fun",
            mods: &[
                OnTheBeat
            ]
        },
    ],

    diff_values: &[
        OVERALL_DIFFICULTY,
        APPROACH_DIFFICULTY,
        CIRCLE_SIZE_DIFFICULTY,
        
        BPM_DIFF_VALUE,
        DURATION_DIFF_VALUE,
    ],

    judgments: OsuHitJudgments::variants(),
    calc_acc: OsuGameInfo::calc_acc,
    create_game: OsuGameInfo::create_game,
    create_diffcalc: OsuGameInfo::create_diffcalc,
    can_load_beatmap: OsuGameInfo::can_load_beatmap,

    
    serialize_settings: OsuGameInfo::serialize_settings,
    deserialize_settings: OsuGameInfo::deserialize_settings,

    .. GamemodeInfo::DEFAULT
};

pub struct OsuGameInfo;
impl OsuGameInfo {
    fn calc_acc(score: &Score) -> f32 {
        let x50  = score.judgments.get("x50").copied().unwrap_or_default()  as f32;
        let x100 = score.judgments.get("x100").copied().unwrap_or_default() as f32;
        let x300 = score.judgments.get("x300").copied().unwrap_or_default() as f32;
        let geki = score.judgments.get("geki").copied().unwrap_or_default() as f32;
        let katu = score.judgments.get("katu").copied().unwrap_or_default() as f32;
        let miss = score.judgments.get("xmiss").copied().unwrap_or_default() as f32;
    
        (50.0 * x50 + 100.0 * (x100 + katu) + 300.0 * (x300 + geki)) 
        / (300.0 * (miss + x50 + x100 + x300 + katu + geki))
    }

    fn can_load_beatmap(map: &BeatmapType) -> bool { 
        matches!(map, BeatmapType::Osu)
    }

    fn create_game<'a>(beatmap: &'a Beatmap, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn GameMode>>> {
        Box::pin(async {
            let game:Box<dyn GameMode> = Box::new(OsuGame::new(beatmap, false, settings)?);
            Ok(game)
        })
    }
    fn create_diffcalc<'a>(map: &'a BeatmapMeta, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn DiffCalc>>> {
        Box::pin(async {
            let calc:Box<dyn DiffCalc> = Box::new(OsuDifficultyCalculator::new(map, settings).await?);
            Ok(calc)
        })
    }


    fn deserialize_settings(value: serde_json::Value) -> Option<Box<dyn GamemodeSettings>> {
        if value.is_null() {
            let settings: Box<dyn GamemodeSettings> = Box::new(OsuSettings::default());
            return Some(settings);
        }

        let parsed = serde_json::from_value::<OsuSettings>(value).ok()?; 
        let a: Box<dyn GamemodeSettings> = Box::new(parsed);
        Some(a)
    }
    fn serialize_settings(s: Box<dyn GamemodeSettings>) -> serde_json::Value {
        let s = *s.downcast::<OsuSettings>().unwrap();
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
    get_diff_value: |map, mods| OsuGame::get_od(map, mods),
};

pub const APPROACH_DIFFICULTY: DifficultyValue = DifficultyValue {
    id: "ar",
    name: "AR",
    modifiable: true,
    number_type: DifficultyNumberType::Float,
    min: 0.0,
    max: 11.0,
    step: Some(0.1),
    unit: None,
    get_diff_value: |map, mods| OsuGame::get_ar(map, mods),
};

pub const CIRCLE_SIZE_DIFFICULTY: DifficultyValue = DifficultyValue {
    id: "cs",
    name: "CS",
    modifiable: true,
    number_type: DifficultyNumberType::Float,
    min: 0.0,
    max: 10.0,
    step: Some(0.1),
    unit: None,
    get_diff_value: |map, mods| OsuGame::get_cs(map, mods),
};

// pub const HEALTH_DIFFICULTY: DifficultyValue = DifficultyValue {
//     id: "hp",
//     name: "HP",
//     modifiable: true,
//     number_type: DifficultyNumberType::Float,
//     min: 0.0,
//     max: 10.0,
//     step: Some(0.1),
//     unit: None,
//     get_diff_value: |map, mods| OsuGame::get_hp(map, mods),
// };