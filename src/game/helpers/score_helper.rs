/*
 * this is a helper to store and retrieve scores, either online or local
 */

use crate::prelude::*;
use ayyeve_piston_ui::prelude::Dropdown;


lazy_static::lazy_static! {
    pub static ref SCORE_HELPER:Arc<RwLock<ScoreHelper>> = Arc::new(RwLock::new(ScoreHelper::new()));
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

    pub fn get_scores(&self, map_hash: &String, playmode: &PlayMode) -> Arc<RwLock<ScoreLoaderHelper>> {
        let map_hash = map_hash.clone();
        let playmode = playmode.clone();
        let method = self.current_method;

        match method {
            ScoreRetreivalMethod::Local 
            | ScoreRetreivalMethod::LocalMods => {
                let scores = Arc::new(RwLock::new(ScoreLoaderHelper::new()));
                let scores_clone = scores.clone();
                tokio::spawn(async move {
                    let mut local_scores = get_scores(&map_hash, playmode);

                    if method.filter_by_mods() {
                        let mods = ModManager::get().clone();
                        let mods_string = Some(serde_json::to_string(&mods).unwrap());
                        local_scores.retain(|s| s.mods_string == mods_string);
                    }
                    let mut thing = scores_clone.write();
                    thing.scores = local_scores;
                    thing.done = true;
                });
                
                scores
            },
            ScoreRetreivalMethod::Global
            | ScoreRetreivalMethod::GlobalMods => {
                let scores = Arc::new(RwLock::new(ScoreLoaderHelper::new()));
                
                let scores_clone = scores.clone();
                tokio::spawn(async move {
                //     let mut online_scores = get_online_scores(&map_hash, playmode);

                //     if method.filter_by_mods() {
                //         let mods = ModManager::get().clone();
                //         let mods_string = Some(serde_json::to_string(&mods).unwrap());
                //         online_scores.retain(|s| s.mods_string == mods_string);
                //     }

                    let mut thing = scores_clone.write();
                //     thing.scores = local_scores;
                    thing.done = true;
                });
                
                //TODO: this
                
                scores.write().done = true;
                scores
            },

            ScoreRetreivalMethod::OgGame
            | ScoreRetreivalMethod::OgGameMods => {
                let scores = Arc::new(RwLock::new(ScoreLoaderHelper::new()));
                
                let scores_clone = scores.clone();
                let map_by_hash = BEATMAP_MANAGER.read().get_by_hash(&map_hash).clone();
                tokio::spawn(async move {
                    let mut online_scores = Vec::new();
                    if let Some(map) = map_by_hash {
                        match map.beatmap_type {
                            BeatmapType::Osu => {
                                let mode = match &*map.check_mode_override(playmode.clone()) {
                                    "osu" => 0,
                                    "taiko" => 1,
                                    "catch" => 2,
                                    "mania" => 3,
                                    _ => panic!("osu how?")
                                };

                                let key = get_settings!().osu_api_key.clone();
                                if key.is_empty() {
                                    NotificationManager::add_text_notification("You need to supply an osu api key in settings.json", 5000.0, Color::RED);
                                } else {
                                    // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
                                    if let Some(id) = osu::fetch_beatmap_id(&key, &map_hash).await {
                                        let url = format!("https://osu.ppy.sh/api/get_scores?k={key}&b={id}&m={mode}");

                                        if let Ok(res) = reqwest::get(url).await {
                                            if let Ok(bytes) = res.bytes().await {
                                                let bytes = bytes.to_vec();
                                                online_scores = osu::scores_from_api_response(bytes, &playmode);
                                            }
                                        }
                                    }
                                }
                            },
                            BeatmapType::Quaver => {
                                // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
                                if let Some(id) = quaver::fetch_beatmap_id(&map_hash).await {
                                    let url = format!("https://api.quavergame.com/v1/scores/map/{id}");

                                    if let Ok(res) = reqwest::get(url).await {
                                        if let Ok(bytes) = res.bytes().await {
                                            let bytes = bytes.to_vec();
                                            online_scores = quaver::scores_from_api_response(bytes);
                                        }
                                    }
                                }
                            },


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

                    let mut thing = scores_clone.write();
                    thing.scores = online_scores;
                    thing.done = true;
                });
                
                scores
            },
        }
    }
}

/// helper for retreiving scores from online (async)
pub struct ScoreLoaderHelper {
    pub scores: Vec<Score>,
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

#[derive(Copy, Clone, PartialEq, Eq, Debug, Dropdown)]
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
    
    pub async fn fetch_beatmap_id(api_key: &String, map_hash: &String) -> Option<String> {
        let url = format!("https://osu.ppy.sh/api/get_beatmaps?k={api_key}&h={map_hash}");
        println!("osu beatmap id lookup");
        let bytes = reqwest::get(url).await.ok()?.bytes().await.ok()?.to_vec();
        let maps: Vec<OsuApiBeatmap> = serde_json::from_slice(bytes.as_slice()).ok()?;
        if let Some(map) = maps.first() {
            Some(map.beatmap_id.clone())
        } else {
            None
        }
    }

    pub fn scores_from_api_response(resp: Vec<u8>, playmode: &PlayMode) -> Vec<Score> {
        let osu_scores:Vec<OsuApiScore> = serde_json::from_slice(resp.as_slice()).unwrap_or_default();

        osu_scores.iter().map(|s| {
            let mut s = Score { 
                version: 0, 
                username: s.username.clone(), 
                beatmap_hash: String::new(), 
                playmode: playmode.clone(), 
                score: s.score.parse().unwrap_or_default(), 
                combo: s.maxcombo.parse().unwrap_or_default(), 
                max_combo: s.maxcombo.parse().unwrap_or_default(), 
                x50: s.count50.parse().unwrap_or_default(), 
                x100: s.count100.parse().unwrap_or_default(), 
                x300: s.count300.parse().unwrap_or_default(), 
                xgeki: s.countgeki.parse().unwrap_or_default(),
                xkatu: s.countkatu.parse().unwrap_or_default(), 
                xmiss: s.countmiss.parse().unwrap_or_default(), 
                accuracy: 1.0,
                speed: 1.0, 
                mods_string: None, // TODO: 
                hit_timings: Vec::new(), 
                replay_string: None
            };
            
            s.accuracy = calc_acc(&s);
            s
        }).collect()
    }

}

mod quaver {
    use crate::prelude::*;

    pub async fn fetch_beatmap_id(map_hash: &String) -> Option<u32> {
        let url = format!("https://api.quavergame.com/v1/maps/{map_hash}");
        let bytes = reqwest::get(url).await.ok()?.bytes().await.ok()?.to_vec();
        let resp:QuaverResponse = serde_json::from_slice(bytes.as_slice()).ok()?;

        if let Some(map) = resp.map {
            Some(map.id)
        } else {
            None
        }
    }
    
    pub fn scores_from_api_response(resp: Vec<u8>) -> Vec<Score> {
        let resp:QuaverResponse = serde_json::from_slice(resp.as_slice()).unwrap();
        if let Some(scores) = resp.scores {
            scores.iter().map(|s| {
                Score { 
                    version: 0, 
                    username: s.user.username.clone(), 
                    beatmap_hash: String::new(), 
                    playmode: String::new(), 
                    score: s.total_score, 
                    combo: s.max_combo as u16, 
                    max_combo: s.max_combo as u16, 
                    x50  : s.count_okay as u16,
                    x100 : s.count_good as u16,
                    x300 : s.count_marv as u16,
                    xgeki: s.count_perf as u16,
                    xkatu: s.count_great as u16,
                    xmiss: s.count_miss as u16,
                    accuracy: s.accuracy as f64 / 100.0,
                    speed: 1.0, 
                    mods_string: Some(s.mods_string.clone()),
                    hit_timings: Vec::new(), 
                    replay_string: None
                }
            }).collect()
        } else {
            Vec::new()
        }
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
