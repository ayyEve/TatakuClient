/*
 * this is a helper to store and retrieve scores, either online or local
 */

use crate::prelude::*;


lazy_static::lazy_static! {
    pub static ref SCORE_HELPER:Arc<AsyncRwLock<ScoreHelper>> = Arc::new(AsyncRwLock::new(ScoreHelper::new()));
}

pub struct ScoreHelper {
    pub current_method: ScoreRetreivalMethod,
}
impl ScoreHelper {
    pub fn new() -> Self {
        Self {
            current_method: ScoreRetreivalMethod::Local,
        }
    }

    pub async fn get_scores(&self, map_hash: &String, playmode: &PlayMode) -> Arc<AsyncRwLock<ScoreLoaderHelper>> {
        let map_hash = map_hash.clone();
        let playmode = playmode.clone();
        let method = self.current_method;
        let scores = Arc::new(AsyncRwLock::new(ScoreLoaderHelper::new()));
        let scores_clone = scores.clone();

        match method {
            ScoreRetreivalMethod::Local 
            | ScoreRetreivalMethod::LocalMods => {
                tokio::spawn(async move {
                    let mut local_scores = Database::get_scores(&map_hash, playmode).await;

                    if method.filter_by_mods() {
                        let mods = ModManager::get().mods.clone();
                        local_scores.retain(|s| s.mods() == mods);
                    }
                    
                    let mut thing = scores_clone.write().await;
                    thing.scores = local_scores.into_iter().map(|s|IngameScore::new(s, false, false)).collect();
                    thing.done = true;
                });
            }
            ScoreRetreivalMethod::Global
            | ScoreRetreivalMethod::GlobalMods => {
                tokio::spawn(async move {
                    let mut online_scores = tataku::get_scores(&map_hash, &playmode).await;

                    if method.filter_by_mods() {
                        let mods = ModManager::get().mods.clone();
                        online_scores.retain(|s| s.mods() == mods);
                    }

                    let mut thing = scores_clone.write().await;
                    thing.scores = online_scores;
                    thing.done = true;
                });
                
                // scores.write().await.done = true;
            }

            ScoreRetreivalMethod::OgGame
            | ScoreRetreivalMethod::OgGameMods => {
                tokio::spawn(async move {
                    let map_by_hash = BEATMAP_MANAGER.read().await.get_by_hash(&map_hash).clone();

                    let mut online_scores = Vec::new();
                    if let Some(map) = map_by_hash {
                        match map.beatmap_type {
                            BeatmapType::Osu => online_scores = osu::get_scores(&map, &playmode).await,
                            BeatmapType::Quaver => online_scores = quaver::get_scores(&map_hash).await,
                            //TODO: add tataku once its implemented


                            BeatmapType::Stepmania
                            | BeatmapType::Tja
                            | BeatmapType::UTyping
                            | BeatmapType::Adofai 
                            | BeatmapType::Unknown => {},
                        }
                    }


                    // if method.filter_by_mods() {
                    //     let mods = ModManager::get().clone();
                    //     let mods_string = Some(serde_json::to_string(&mods).unwrap());
                    //     // online_scores.retain(|s| s.mods_string == mods_string);
                    // }

                    let mut thing = scores_clone.write().await;
                    thing.scores = online_scores;
                    thing.done = true;
                });
            }
        }

        scores
    }
}

/// helper for retreiving scores from online (async)
pub struct ScoreLoaderHelper {
    pub scores: Vec<IngameScore>,
    pub done: bool
}
impl ScoreLoaderHelper {
    pub fn new() -> Self {
        Self {
            scores: Vec::new(),
            done: false
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Dropdown, Serialize, Deserialize)]
pub enum ScoreRetreivalMethod {
    Local,
    LocalMods,
    Global,
    GlobalMods,

    OgGame,
    OgGameMods,
    // Friends,
    // FriendsMods
}
impl ScoreRetreivalMethod {
    pub fn filter_by_mods(&self) -> bool {
        use ScoreRetreivalMethod::*;
        match self {
            Local 
            | OgGame 
            // | Friends
            | Global => false,

            LocalMods
            // | FriendsMods
            | OgGameMods
            | GlobalMods => true,
        }
    }
}


//TODO: use the api crates?

mod osu {
    use crate::prelude::*;
    
    #[derive(Serialize, Deserialize)]
    struct OsuApiScore {
        score_id: String,
        score: String,
        username: String,
        maxcombo: String,
        count300: String,
        count100: String,
        count50: String,
        countmiss: String,
        countgeki: String,
        countkatu: String,
        perfect: String,
        enabled_mods: String,
        user_id: String,
        date: String,
        rank: String,
        pp: String,
        replay_available: String,
    }

    #[derive(Serialize, Deserialize)]
    struct OsuApiBeatmap {
        beatmapset_id: String,
        beatmap_id: String,
        // dont care about anything else for this
    }
    
    struct Mods;
    #[allow(non_upper_case_globals, unused)]
    impl Mods {
        const None:u64        = 0;
        const NoFail:u64      = 1;
        const Easy:u64        = 2;
        const TouchDevice:u64 = 4;
        const Hidden:u64      = 8;
        const HardRock:u64    = 16;
        const SuddenDeath:u64 = 32;
        const DoubleTime:u64  = 64;
        const Relax:u64       = 128;
        const HalfTime:u64    = 256;
        const Nightcore:u64   = 512;
        const Flashlight:u64  = 1024;
        const Autoplay:u64    = 2048;
        const SpunOut:u64     = 4096;
        const Autopilot:u64   = 8192;
        const Perfect:u64     = 16384;
        const Key4:u64        = 32768;
        const Key5:u64        = 65536;
        const Key6:u64        = 131072;
        const Key7:u64        = 262144;
        const Key8:u64        = 524288;
        const FadeIn:u64      = 1048576;
        const Random:u64      = 2097152;
        const Cinema:u64      = 4194304;
        const Target:u64      = 8388608;
        const Key9:u64        = 16777216;
        const KeyCoop:u64     = 33554432;
        const Key1:u64        = 67108864;
        const Key3:u64        = 134217728;
        const Key2:u64        = 268435456;
        const ScoreV2:u64     = 536870912;
        const Mirror:u64      = 1073741824;
    }


    pub async fn fetch_beatmap_id(api_key: &String, map_hash: &String) -> Option<String> {
        let url = format!("https://osu.ppy.sh/api/get_beatmaps?k={api_key}&h={map_hash}");
        trace!("osu beatmap id lookup");
        let bytes = reqwest::get(url).await.ok()?.bytes().await.ok()?.to_vec();
        let maps: Vec<OsuApiBeatmap> = serde_json::from_slice(bytes.as_slice()).ok()?;

        maps.first().map(|m|m.beatmap_id.clone())
    }

    pub async fn get_scores(map: &Arc<BeatmapMeta>, playmode: &String) -> Vec<IngameScore> {
        match get_scores_internal(map, playmode).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting osu scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(map: &Arc<BeatmapMeta>, playmode: &String) -> TatakuResult<Vec<IngameScore>> {
        let mode = match &*map.check_mode_override(playmode.clone()) {
            "osu" => 0,
            "taiko" => 1,
            "catch" => 2,
            "mania" => 3,
            _ => panic!("osu how?")
        };

        let key = Settings::get().osu_api_key.clone();
        if key.is_empty() {
            NotificationManager::add_text_notification("You need to supply an osu api key in settings.json", 5000.0, Color::RED).await;
            Err(TatakuError::String("no api key".to_owned()))
        } else {
            // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
            if let Some(id) = fetch_beatmap_id(&key, &map.beatmap_hash).await {
                let url = format!("https://osu.ppy.sh/api/get_scores?k={key}&b={id}&m={mode}");

                let bytes = reqwest::get(url).await?.bytes().await?;
                let bytes = bytes.to_vec();
                let osu_scores:Vec<OsuApiScore> = serde_json::from_slice(bytes.as_slice()).unwrap_or_default();

                Ok(osu_scores.iter().map(|s| {

                    let mut judgments = HashMap::new();
                    judgments.insert("x50".to_owned(),  s.count50.parse().unwrap_or_default());
                    judgments.insert("x100".to_owned(),  s.count100.parse().unwrap_or_default());
                    judgments.insert("x300".to_owned(),  s.count300.parse().unwrap_or_default());
                    judgments.insert("xgeki".to_owned(), s.countgeki.parse().unwrap_or_default());
                    judgments.insert("xkatu".to_owned(), s.countkatu.parse().unwrap_or_default());
                    judgments.insert("xmiss".to_owned(), s.countmiss.parse().unwrap_or_default());


                    let mut score = Score::default();
                    score.username = s.username.clone();
                    score.playmode = playmode.clone();
                    score.score = s.score.parse().unwrap_or_default();
                    score.combo = s.maxcombo.parse().unwrap_or_default();
                    score.max_combo = s.maxcombo.parse().unwrap_or_default();
                    score.judgments = judgments;
                    score.speed = 1.0;
                    score.accuracy = calc_acc(&score);

                    // mods
                    {
                        let peppy_fuck = s.enabled_mods.parse::<u64>().unwrap_or_default();
                        macro_rules! check { 
                            ($i: ident, $n: expr) => { 
                                if (peppy_fuck & Mods::$i) > 0 { score.mods_mut().insert($n.to_owned()); } 
                            }; 
                        }

                        check!(NoFail, "no_fail");
                        check!(Easy, "easy");
                        check!(Hidden, "hidden");
                        check!(HardRock, "hard_rock");
                        check!(SuddenDeath, "sudden_death");
                        check!(Relax, "relax");
                        check!(Flashlight, "flash_light");
                        check!(Autoplay, "autoplay");
                        check!(Autopilot, "auto_pilot");
                        check!(SpunOut, "spun_out");
                        check!(Perfect, "perfect");
                        // mania mods
                        check!(FadeIn, "fade_in");
                        check!(Random, "random");
                        check!(Mirror, "mirror");

                        if (peppy_fuck & Mods::DoubleTime) > 0 { score.speed = 1.5; }
                        if (peppy_fuck & Mods::HalfTime) > 0 { score.speed = 0.75; }
                    }

                    let mut score = IngameScore::new(score, false, false);
                    // error!("{}", s.replay_available);
                    score.replay_location = if s.replay_available == "1" {
                        ReplayLocation::Online(Arc::new(OsuReplayDownloader::new(score.score.clone(), id.parse().unwrap_or_default())))
                    } else {
                        ReplayLocation::OnlineNotExist
                    };

                    score
                }).collect())
                    
            } else {
                Err(TatakuError::String("no osu map".to_owned()))
            }
        }
    }

}

mod quaver {
    use crate::prelude::*;

    pub async fn fetch_beatmap_id(map_hash: &String) -> Option<u32> {
        let url = format!("https://api.quavergame.com/v1/maps/{map_hash}");
        let bytes = reqwest::get(url).await.ok()?.bytes().await.ok()?;
        let resp:QuaverResponse = serde_json::from_slice(&bytes).ok()?;

        resp.map.map(|m|m.id)
    }
    

    pub async fn get_scores(map_hash: &String) -> Vec<IngameScore> {
        match get_scores_internal(map_hash).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting quaver scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(map_hash: &String) -> TatakuResult<Vec<IngameScore>> {

        // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
        let Some(id) = fetch_beatmap_id(map_hash).await else {return Err(TatakuError::String("no osu map".to_owned()))};
        let url = format!("https://api.quavergame.com/v1/scores/map/{id}");

        let bytes = reqwest::get(url).await?.bytes().await?;
        // online_scores = quaver::scores_from_api_response(bytes);

        let resp:QuaverResponse = serde_json::from_slice(&bytes)?;

        Ok(resp.scores.unwrap_or_default().iter().map(|s| {
            let mut judgments = HashMap::new();
            judgments.insert("x50".to_owned(),   s.count_okay as u16);
            judgments.insert("x100".to_owned(),  s.count_good as u16);
            judgments.insert("x300".to_owned(),  s.count_marv as u16);
            judgments.insert("xgeki".to_owned(), s.count_perf as u16);
            judgments.insert("xkatu".to_owned(), s.count_great as u16);
            judgments.insert("xmiss".to_owned(), s.count_miss as u16);


            let mut score = Score::default();
            score.username = s.user.username.clone();
            score.score = s.total_score;
            score.combo = s.max_combo as u16;
            score.max_combo = s.max_combo as u16;
            score.judgments = judgments;
            score.speed = 1.0;
            score.accuracy = s.accuracy as f64 / 100.0;
            // check mods
            for m in s.mods_string.split(", ") {
                if m.ends_with("x") {
                    if let Ok(speed) = m.trim_end_matches("x").parse() {
                        score.speed = speed;
                        continue;
                    }
                }

                score.mods_mut().insert(m.to_lowercase());
            }
            

            let mut score = IngameScore::new(score, false, false);
            score.replay_location = ReplayLocation::Online(Arc::new(QuaverReplayDownloader::new(score.score.clone(), s.id)));

            score
        }).collect())
    }

    // helper because im lazy
    #[derive(Serialize, Deserialize)]
    struct QuaverResponse {
        status: u16,
        map: Option<QuaverApiBeatmap>,
        scores: Option<Vec<QuaverApiScore>>
    }

    /// https://wiki.quavergame.com/docs/api/maps
    #[derive(Serialize, Deserialize)]
    struct QuaverApiBeatmap {
        id: u32,
        mapset_id: u32,

        // dont care about anything else

    }

    /// https://wiki.quavergame.com/docs/api/scores
    #[derive(Serialize, Deserialize)]
    struct QuaverApiScore {
        id: u32,
        map_md5: String,
        time: String,
        mode: u8,
        mods: u64,
        mods_string: String,
        performance_rating: f64,
        total_score: u64,
        accuracy: f32,
        grade: String,
        
        max_combo: u32,
        count_marv: u32,
        count_perf: u32,
        count_great: u32,
        count_good: u32,
        count_okay: u32,
        count_miss: u32,

        user: QuaverApiBeatmapUser
    }

    #[derive(Serialize, Deserialize)]
    struct QuaverApiBeatmapUser {
        id: u32,
        username: String,
        country: String,
        avatar_url: String,
        // dont care about anything else
    }

}


mod tataku {
    use crate::prelude::*;

    #[derive(Serialize, Deserialize)]
    struct TatakuScore {
        score_id: u64,
        score_hash: Option<String>,
        score: Score
    }

    pub async fn get_scores(map_hash: &String, playmode: &String) -> Vec<IngameScore> {
        match get_scores_internal(map_hash, playmode).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting tataku scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(map_hash: &String, playmode: &String) -> TatakuResult<Vec<IngameScore>> {
        let base = Settings::get().score_url.clone();
        let url = format!("{base}/api/get_scores?hash={map_hash}&mode={playmode}");

        let bytes = reqwest::get(url).await?.bytes().await?.to_vec();
        let maps: Vec<TatakuScore> = serde_json::from_slice(bytes.as_slice())?;

        Ok(maps.into_iter().map(|s| {
            let mut score = IngameScore::new(s.score, false, false);
            if s.score_hash.is_none() {
                score.replay_location = ReplayLocation::OnlineNotExist;
            } else {
                score.replay_location = ReplayLocation::Online(Arc::new(TatakuReplayDownloader::new(s.score_id, s.score_hash)));
            }
            score
        }).collect())
    }

}
