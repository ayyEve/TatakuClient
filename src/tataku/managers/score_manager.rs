use crate::prelude::*;
use tokio::task::AbortHandle;


#[derive(Reflect)]
#[derive(Debug)]
pub struct ScoreManager {
    pub scores: Vec<IngameScore>,
    pub loaded: bool,

    #[reflect(skip)]
    current_loader: Option<Arc<AsyncRwLock<ScoreLoaderHelper>>>,
    #[reflect(skip)]
    abort_handle: Option<AbortHandle>,

    pub force_update: bool,

    #[reflect(skip)]
    beatmap: SyValueHelper<Md5Hash>,
    #[reflect(skip)]
    playmode: SyValueHelper<String>,
    #[reflect(skip)]
    score_method: SyValueHelper<ScoreRetreivalMethod>,
    #[reflect(skip)]
    mods: SyValueHelper<ModManager>,
}
impl ScoreManager {
    pub fn new() -> Self {
        Self {
            scores: Vec::new(),
            loaded: false,

            current_loader: None,
            abort_handle: None,
            force_update: false,

            beatmap: SyValueHelper::new("beatmaps.current.hash"),
            playmode: SyValueHelper::new("global.playmode_actual"),
            score_method: SyValueHelper::new("settings.score_method"),
            mods: SyValueHelper::new("global.mods"),
        }
    }

    fn check_mods(score_mods: &Vec<ModDefinition>, mod_manager: &ModManager) -> bool {
        if score_mods.len() != mod_manager.mods.len() { return false }

        for i in score_mods.iter() {
            if !mod_manager.has_mod(i.as_ref()) { return false }
        }

        true
    }


    pub async fn get_scores(&mut self, values: &mut ValueCollection) -> TatakuResult {
        if self.current_loader.take().is_some() {
            if let Some(abort) = self.abort_handle.take() {
                abort.abort();
            }
        }   

        // let playmode = values.get_string("global.playmode_actual").ok()?;
        // let map_hash = values.try_get::<Md5Hash>("map.hash").ok()?;
        // let method = values.try_get("settings.score_method").unwrap_or_default();
        let playmode = self.playmode.try_get()?.clone();
        let map_hash:Md5Hash = *self.beatmap.try_get()?;
        let method = self.score_method();

        let scores = Arc::new(AsyncRwLock::new(ScoreLoaderHelper::default()));
        self.current_loader = Some(scores.clone());
        let scores_clone = scores.clone();
        
        match self.score_method() {
            ScoreRetreivalMethod::Local 
            | ScoreRetreivalMethod::LocalMods => {
                let mods = self.mods.as_ref().cloned().unwrap_or_default();

                let handle = tokio::spawn(async move {
                    let map_hash = map_hash.to_string();
                    let mut local_scores = Database::get_scores(&map_hash, playmode).await;

                    if method.filter_by_mods() {
                        local_scores.retain(|s| Self::check_mods(&s.mods, &mods));
                    }
                    
                    let mut thing = scores_clone.write().await;
                    thing.scores = local_scores.into_iter().map(|s| IngameScore::new(s, false, false)).collect();
                    thing.done = true;
                });

                self.abort_handle = Some(handle.abort_handle());
            }
            ScoreRetreivalMethod::Global
            | ScoreRetreivalMethod::GlobalMods => {
                let mods = self.mods.as_ref().cloned().unwrap_or_default();
                // let beatmap_type = values.try_get::<BeatmapType>("map.beatmap_type")?;

                let handle = tokio::spawn(async move {
                    let map_hash = map_hash.to_string();
                    let mut online_scores = tataku::get_scores(&map_hash, &playmode).await;

                    if method.filter_by_mods() {
                        online_scores.retain(|s| Self::check_mods(&s.mods, &mods));
                    }

                    let mut thing = scores_clone.write().await;
                    thing.scores = online_scores;
                    thing.done = true;
                });

                self.abort_handle = Some(handle.abort_handle());
            }

            ScoreRetreivalMethod::OgGame
            | ScoreRetreivalMethod::OgGameMods => {
                let beatmap_type = values
                    .beatmap_manager
                    .current_beatmap
                    .as_ref()
                    .map(|b| b.beatmap_type)
                    .ok_or(TatakuError::String(format!("no beatmap")))?;
                
                let osu_api_key = values.settings.osu_api_key.clone();

                let handle = tokio::spawn(async move {
                    let mut online_scores = Vec::new();
                    match beatmap_type {
                        BeatmapType::Osu => online_scores = osu::get_scores(
                            &osu_api_key,
                            map_hash, 
                            &playmode
                        ).await,
                        BeatmapType::Quaver => online_scores = quaver::get_scores(map_hash).await,
                        //TODO: add tataku once its implemented


                        BeatmapType::Stepmania
                        | BeatmapType::Tja
                        | BeatmapType::UTyping
                        | BeatmapType::Adofai 
                        | BeatmapType::Unknown => {},
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

                self.abort_handle = Some(handle.abort_handle());
            }
        }

        Ok(())
    }

    fn score_method(&self) -> ScoreRetreivalMethod {
        self.score_method.as_ref().copied().unwrap_or_default()
    }

    // fn update_values(&self, values: &mut ValueCollection, loaded: bool) {
    //     let list = self.current_scores.iter().enumerate().map(|(n, score)| {
    //         let score:TatakuValue = score.into();
    //         let mut data = score.to_map();
    //         data.set_value("id", TatakuVariable::new(n as u64));

    //         TatakuVariable::new_game(data)
    //     }).collect::<Vec<_>>();

    //     let mut score_list = HashMap::default();
    //     score_list.set_value("loaded", TatakuVariable::new_game(loaded));
    //     score_list.set_value("empty", TatakuVariable::new_game(list.is_empty()));
    //     score_list.set_value("scores", TatakuVariable::new_game(TatakuValue::List(list)));
    //     values.set("score_list", TatakuVariable::new_game(score_list));
    // }
    
    pub async fn update(&mut self, values: &mut ValueCollection) {
        let did_update = 
            self.beatmap.update(values).unwrap().is_some() // if the map changed
            | self.playmode.update(values).unwrap().is_some() // or the actual playmode changed
            | self.score_method.update(values).unwrap().is_some() // or the score method changed
            | (self.mods.update(values).unwrap().is_some() && self.score_method().filter_by_mods()) // or the mods changed and the score method filters by mods
            | self.force_update
            ;

        if did_update {
            trace!("doing score update");
            self.force_update = false;

            // clear scores and update values
            self.scores.clear();
            self.loaded = false;

            // and then get new scores
            if let Err(e) = self.get_scores(values).await {
                warn!("error getting scores: {e}");
            }
        }

        if let Some(loader) = self.current_loader.clone() {
            if let Ok(loader) = loader.try_read() {
                if !loader.done { return } 

                self.scores = loader.scores.clone();
                self.current_loader = None;
                self.abort_handle = None;
                self.loaded = true;
            }
        }

    }


    pub fn get_score(&self, id: usize) -> Option<&IngameScore> {
        self.scores.get(id)
    }
}

impl Default for ScoreManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
#[derive(Debug)]
pub struct ScoreLoaderHelper {
    pub scores: Vec<IngameScore>,
    pub done: bool,
}



#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[derive(Reflect)]
pub enum ScoreRetreivalMethod {
    #[default]
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
    pub fn list() -> Vec<Self> {
        vec![
            Self::Local,
            Self::LocalMods,
            
            Self::Global,
            Self::GlobalMods,

            Self::OgGame,
            Self::OgGameMods,
        ]
    }

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
impl Display for ScoreRetreivalMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl TryFrom<&TatakuValue> for ScoreRetreivalMethod {
    type Error = String;
    fn try_from(value: &TatakuValue) -> Result<Self, Self::Error> {
        match value {
            TatakuValue::String(s) => {
                match &**s {
                    "Local" | "local" => Ok(Self::Local),
                    "LocalMods" | "local_mods" => Ok(Self::LocalMods),

                    "Global" | "global" => Ok(Self::Global),
                    "GlobalMods" | "global_mods" => Ok(Self::GlobalMods),

                    "OgGame" | "og_game" => Ok(Self::OgGame),
                    "OgGameMods" | "og_game_mods" => Ok(Self::OgGameMods),

                    other => Err(format!("invalid ScoreRetreivalMethod str: '{other}'"))
                }
            }
            TatakuValue::U64(n) => {
                match *n {
                    0 => Ok(Self::Local),
                    1 => Ok(Self::LocalMods),
                    2 => Ok(Self::Global),
                    3 => Ok(Self::GlobalMods),
                    4 => Ok(Self::OgGame),
                    5 => Ok(Self::OgGameMods),
                    other => Err(format!("Invalid ScoreRetreivalMethod number: {other}")),
                }
            }

            other => Err(format!("Invalid ScoreRetreivalMethod value: {other:?}"))
        }
    }
}
impl Into<TatakuValue> for ScoreRetreivalMethod {
    fn into(self) -> TatakuValue {
        TatakuValue::String(format!("{self:?}"))
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

    pub async fn get_scores(
        osu_api_key: &String,
        hash: Md5Hash,
        playmode: &String
    ) -> Vec<IngameScore> {
        match get_scores_internal(osu_api_key, hash, playmode).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting osu scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(
        osu_api_key: &String,
        hash: Md5Hash,
        playmode: &String
    ) -> TatakuResult<Vec<IngameScore>> {
        let ok_mods = ModManager::mods_for_playmode_as_hashmap(playmode);

        let mode = match &**playmode {
            "osu" => 0,
            "taiko" => 1,
            "catch" => 2,
            "mania" => 3,
            _ => return Err(TatakuError::Beatmap(BeatmapError::UnsupportedMode))
        };

        // let key = Settings::get().osu_api_key.clone();
        if osu_api_key.is_empty() {
            NotificationManager::add_text_notification("You need to supply an osu api key in settings.json", 5000.0, Color::RED).await;
            Err(TatakuError::String("no api key".to_owned()))
        } else {
            let hash = hash.to_string();
            // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
            if let Some(id) = fetch_beatmap_id(&osu_api_key, &hash).await {
                let url = format!("https://osu.ppy.sh/api/get_scores?k={osu_api_key}&b={id}&m={mode}");

                let bytes = reqwest::get(url).await?.bytes().await?;
                let bytes = bytes.to_vec();
                let osu_scores:Vec<OsuApiScore> = serde_json::from_slice(bytes.as_slice()).unwrap_or_default();

                Ok(osu_scores.iter().map(|s| {

                    let mut judgments = HashMap::new();
                    judgments.insert("x50".to_owned(),   s.count50.parse().unwrap_or_default());
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
                    score.speed = GameSpeed::default();
                    score.accuracy = calc_acc(&score);

                    // mods
                    {
                        let peppy_fuck = s.enabled_mods.parse::<u64>().unwrap_or_default();
                        macro_rules! check { 
                            ($i: ident, $n: expr) => { 
                                if (peppy_fuck & Mods::$i) > 0 { 
                                    if let Some(m) = ok_mods.get($n) {
                                        score.mods.push((*m).into()); 
                                    }
                                } 
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

                        if (peppy_fuck & Mods::DoubleTime) > 0 { score.speed = GameSpeed::from_f32(1.5); }
                        if (peppy_fuck & Mods::HalfTime) > 0 { score.speed = GameSpeed::from_f32(0.75); }
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
    

    pub async fn get_scores(map_hash: Md5Hash) -> Vec<IngameScore> {
        let map_hash = map_hash.to_string();
        match get_scores_internal(&map_hash).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting quaver scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(map_hash: &String) -> TatakuResult<Vec<IngameScore>> {
        let ok_mods = ModManager::mods_for_playmode_as_hashmap("mania");


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
            score.speed = GameSpeed::default();
            score.accuracy = s.accuracy / 100.0;

            // check mods
            for m in s.mods_string.split(", ") {
                if m.ends_with("x") {
                    if let Ok(speed) = m.trim_end_matches("x").parse() {
                        score.speed = GameSpeed::from_f32(speed);
                        continue;
                    }
                }

                if let Some(m) = ok_mods.get(m) {
                    score.mods.push((*m).into()); 
                }

                // score.mods_mut().insert(m.to_lowercase());
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
