use crate::prelude::*;


pub const GAME_INFO: GamemodeInfo = GamemodeInfo {
    id: "utyping",
    display_name: "uTyping",
    about: "utyping",
    author: "ayyEve",

    mods: &[],
    diff_values: &[
        BPM_DIFF_VALUE,
        DURATION_DIFF_VALUE,
    ],

    judgments: UTypingHitJudgment::variants(),
    calc_acc: UTypingGameInfo::calc_acc,
    // get_diff_string: UTypingGameInfo::get_diff_string,
    create_game: UTypingGameInfo::create_game,
    create_diffcalc: UTypingGameInfo::create_diffcalc,
    can_load_beatmap: UTypingGameInfo::can_load_beatmap,

    serialize_settings: UTypingGameInfo::serialize_settings,
    deserialize_settings: UTypingGameInfo::deserialize_settings,

    .. GamemodeInfo::DEFAULT
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

    fn can_load_beatmap(map: &BeatmapType) -> bool { 
        matches!(map, BeatmapType::UTyping)
    }

    fn create_game<'a>(beatmap: &'a Beatmap, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn GameMode>>> {
        Box::pin(async {
            let game:Box<dyn GameMode> = Box::new(UTypingGame::new(beatmap, false, settings).await?);
            Ok(game)
        })
    }
    fn create_diffcalc<'a>(map: &'a BeatmapMeta, settings: &'a Settings) -> BoxFuture<'a, TatakuResult<Box<dyn DiffCalc>>> {
        Box::pin(async {
            let calc:Box<dyn DiffCalc> = Box::new(UTypingDifficultyCalculator::new(map, settings).await?);
            Ok(calc)
        })
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