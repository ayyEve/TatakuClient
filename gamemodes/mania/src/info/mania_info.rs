use crate::prelude::*;

pub const GAME_INFO:GamemodeInfo = GamemodeInfo {
    id: "mania",
    display_name: "Mania",
    about: "mania!",
    author: "ayyEve",

    mods: &[],

    diff_values: &[
        KEYS_DIFF_VALUE,
        BPM_DIFF_VALUE,
        DURATION_DIFF_VALUE,
    ],

    judgments: ManiaHitJudgments::variants(),
    calc_acc: ManiaGameInfo::calc_acc,
    // get_diff_string: ManiaGameInfo::get_diff_string,
    create_game: ManiaGameInfo::create_game,
    create_diffcalc: ManiaGameInfo::create_diffcalc,
    can_load_beatmap: ManiaGameInfo::can_load_beatmap,

    serialize_settings: ManiaGameInfo::serialize_settings,
    deserialize_settings: ManiaGameInfo::deserialize_settings,

    .. GamemodeInfo::DEFAULT
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


    fn can_load_beatmap(map: &BeatmapType) -> bool { 
        matches!(map, BeatmapType::Osu | BeatmapType::Quaver | BeatmapType::Stepmania)
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

    fn deserialize_settings(value: serde_json::Value) -> Option<Box<dyn GamemodeSettings>> {
        if value.is_null() {
            let settings: Box<dyn GamemodeSettings> = Box::new(ManiaSettings::default());
            return Some(settings);
        }

        let parsed = serde_json::from_value::<ManiaSettings>(value).ok()?; 
        let a: Box<dyn GamemodeSettings> = Box::new(parsed);
        Some(a)
    }
    
    fn serialize_settings(s: Box<dyn GamemodeSettings>) -> serde_json::Value {
        let s = *s.downcast::<ManiaSettings>().unwrap();
        serde_json::to_value(&s).unwrap()
    }
}

const KEYS_DIFF_VALUE: DifficultyValue = DifficultyValue {
    id: "keys",
    name: "Keys",
    modifiable: false,
    number_type: DifficultyNumberType::WholeNumber,
    min: 1.0,
    max: 9.0,
    step: None,
    unit: Some("k"),
    get_diff_value: get_key_count,
};
fn get_key_count(map: &BeatmapMetaWithDiff, _: &ModManager) -> f32 {
    map.cs
}