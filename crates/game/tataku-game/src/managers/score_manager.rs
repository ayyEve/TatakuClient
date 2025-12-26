use crate::prelude::*;
use tokio::sync::RwLock;
use tokio::task::AbortHandle;
use common::{
    Score,
    Md5Hash,
    GameSpeed,
    reflect::*,
};

use engine::{
    data::{
        ValueChange,
        ScoreRetreivalMethod,
    },
    beatmaps::BeatmapType,
    gameplay::{
        mods::*,
        IngameScore,
        GamemodeInfos,
        ReplayLocation,
    },
};


#[derive(Reflect)]
#[derive(Clone, Debug)]
pub struct ScoreManager {
    pub force_update: bool,
    pub scores: Vec<IngameScore>,

    #[reflect(skip)] infos: GamemodeInfos,
    #[reflect(skip)] current: Option<Arc<RwLock<Scores>>>,
    #[reflect(skip)] abort_handle: Option<AbortHandle>,

    #[reflect(skip)] beatmap: ValueChange<Md5Hash>,
    #[reflect(skip)] playmode: ValueChange<Arc<str>>,
    #[reflect(skip)] score_method: ValueChange<ScoreRetreivalMethod>,
    #[reflect(skip)] mods: ValueChange<Mods>,
}
impl ScoreManager {
    pub fn new(infos: GamemodeInfos) -> Self {
        Self {
            scores: Vec::new(),
            infos,

            current: None,
            abort_handle: None,
            force_update: false,

            beatmap: ValueChange::new("beatmaps.current"),
            playmode: ValueChange::new("global.playmode_actual"),
            score_method: ValueChange::new("settings.score_method"),
            mods: ValueChange::new("global.mods"),
        }
    }

    fn check_mods(score_mods: &[common::ModDefinition], mods: &Mods) -> bool {
        if score_mods.len() != mods.mods.len() { return false }

        for i in score_mods.iter() {
            if !mods.has_mod(i.as_ref()) { return false }
        }

        true
    }


    pub fn get_scores(
        &mut self,
        values: &mut ValueCollection,
        database: &dyn engine::database::ScoreProvider,
    ) -> engine::tataku::Result<()> {
        if self.current.take().is_some()
        && let Some(abort) = self.abort_handle.take() {
            abort.abort();
        }
        let settings = values.settings.clone();

        let playmode = self.playmode.try_get()?.clone();
        let map_hash = *self.beatmap.try_get()?;
        let method = self.score_method();
        let infos = values.global.gamemode_infos.clone();

        let scores = Arc::new(RwLock::new(Scores::default()));
        self.current = Some(scores.clone());
        let scores_clone = scores.clone();

        match self.score_method() {
            ScoreRetreivalMethod::Local
            | ScoreRetreivalMethod::LocalMods => {
                let mods = self.mods.as_ref().cloned().unwrap_or_default();

                // FIXME: need to async this somehow?
                // let handle = tokio::spawn(async move {
                    let mut local_scores = database.get_scores(
                        map_hash,
                        &playmode,
                        &infos
                    ).unwrap_or_default();

                let handle = tokio::spawn(async move {
                    if method.filter_by_mods() {
                        local_scores.retain(|s| Self::check_mods(&s.mods, &mods));
                    }

                    let mut thing = scores_clone.write().await;
                    thing.scores = local_scores
                        .into_iter()
                        .map(|mut s| {
                            if let Ok(info) = infos.get_info(&s.playmode)
                            && s.accuracy == 0.0 {
                                s.accuracy = info.calc_acc(&s);
                            }

                            s
                        })
                        .map(|s| IngameScore::new(s, false, false))
                        .collect();
                    thing.done = true;
                });

                self.abort_handle = Some(handle.abort_handle());
            }
            ScoreRetreivalMethod::Global
            | ScoreRetreivalMethod::GlobalMods => {
                let mods = self.mods.as_ref().cloned().unwrap_or_default();

                let handle = tokio::spawn(async move {
                    let map_hash = map_hash.to_string();
                    let mut online_scores = tataku::get_scores(
                        &map_hash,
                        &playmode,
                        &settings
                    ).await;

                    if method.filter_by_mods() {
                        online_scores.retain(|s| Self::check_mods(
                            &s.mods,
                            &mods
                        ));
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
                    .current_beatmap_prop(|b| b.beatmap_type)
                    .ok_or("no beatmap")?;

                let osu_api_key = values.settings.integrations.osu.api_key.clone();
                let infos = self.infos.clone();

                let handle = tokio::spawn(async move {
                    let mut online_scores = Vec::new();
                    match beatmap_type {
                        BeatmapType::Osu => online_scores = osu::get_scores(
                            &osu_api_key,
                            map_hash,
                            &playmode,
                            &infos
                        ).await,
                        BeatmapType::Quaver => online_scores = quaver::get_scores(map_hash, &infos).await,
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

    pub fn update(
        &mut self,
        values: &mut ValueCollection,
        database: &dyn engine::database::ScoreProvider
    ) {
        let did_update =
            self.beatmap.update(values).ok().and_then(|a| a).is_some() // if the map changed
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
            // self.loaded = false;
            values.score_list.scores.clear();
            values.score_list.loaded = false;

            // and then get new scores
            if let Err(e) = self.get_scores(values, database) {
                warn!("error getting scores: {e}");
            }
        }

        if let Some(loader) = self.current.clone()
        && let Ok(loader) = loader.try_read() {
            if !loader.done { return }

            let mut scores = loader.scores.clone();
            scores
                .iter_mut()
                .enumerate()
                .for_each(|(n, s)| s.id = n);

            self.current = None;
            self.abort_handle = None;
            self.scores = scores.clone();
            values.score_list.scores = scores;
            values.score_list.loaded = true;
            info!("scores loaded: {:?}", self.scores);
        }

    }


    pub fn get_score(&self, id: usize) -> Option<&IngameScore> {
        self.scores.get(id)
    }
}
impl Default for ScoreManager {
    fn default() -> Self {
        Self::new(GamemodeInfos::default())
    }
}

#[derive(Debug, Default)]
pub struct Scores {
    pub scores: Vec<IngameScore>,
    pub done: bool,
}







// TODO: use the api crates?

mod osu {
    use crate::prelude::*;
    use common::{
        Score,
        Md5Hash,
        GameSpeed,
    };

    use engine::gameplay::{
        IngameScore,
        GamemodeInfos,
        ReplayLocation,
        mods::*,
    };

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

    struct OsuMods;
    #[allow(non_upper_case_globals, unused)]
    impl OsuMods {
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


    pub async fn fetch_beatmap_id(api_key: &str, map_hash: &str) -> Option<String> {
        let url = format!("https://osu.ppy.sh/api/get_beatmaps?k={api_key}&h={map_hash}");
        trace!("osu beatmap id lookup");
        let maps = super::make_request::<Vec<OsuApiBeatmap>>(&url).ok()?;

        maps.first().map(|m|m.beatmap_id.clone())
    }

    pub async fn get_scores(
        osu_api_key: &str,
        hash: Md5Hash,
        playmode: &str,
        infos: &GamemodeInfos,
    ) -> Vec<IngameScore> {
        match get_scores_internal(osu_api_key, hash, playmode, infos).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting osu scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(
        osu_api_key: &str,
        hash: Md5Hash,
        playmode: &str,
        infos: &GamemodeInfos,
    ) -> tataku::Result<Vec<IngameScore>> {
        let info = infos.get_info(playmode)?;
        let ok_mods = Mods::mods_for_playmode_as_hashmap(info);

        let mode = match playmode {
            "osu" => 0,
            "taiko" => 1,
            "catch" => 2,
            "mania" => 3,
            _ => return Err(tataku::Error::Beatmap(errors::beatmap::BeatmapError::UnsupportedMode))
        };

        // let key = Settings::get().osu_api_key.clone();
        if osu_api_key.is_empty() {
            // NotificationManager::add_text_notification("You need to supply an osu api key in settings.json", 5000.0, Color::RED).await;
            Err(tataku::Error::String("no api key".to_owned()))
        } else {
            let hash = hash.to_string();
            // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
            if let Some(id) = fetch_beatmap_id(osu_api_key, &hash).await {
                let url = format!("https://osu.ppy.sh/api/get_scores?k={osu_api_key}&b={id}&m={mode}");

                let osu_scores = super::make_request::<Vec<OsuApiScore>>(&url)
                    .unwrap_or_default();

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
                    score.playmode = playmode.to_string();
                    score.score = s.score.parse().unwrap_or_default();
                    score.combo = s.maxcombo.parse().unwrap_or_default();
                    score.max_combo = s.maxcombo.parse().unwrap_or_default();
                    score.judgments = judgments;
                    score.speed = GameSpeed::default();
                    score.accuracy = info.calc_acc(&score);

                    // mods
                    {
                        let peppy_fuck = s.enabled_mods.parse::<u64>().unwrap_or_default();
                        macro_rules! check {
                            ($i: ident, $n: expr) => {
                                if (peppy_fuck & OsuMods::$i) > 0 {
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

                        if (peppy_fuck & OsuMods::DoubleTime) > 0 { score.speed = GameSpeed::from_f32(1.5); }
                        if (peppy_fuck & OsuMods::HalfTime) > 0 { score.speed = GameSpeed::from_f32(0.75); }
                    }

                    let mut score = IngameScore::new(score, false, false);
                    // error!("{}", s.replay_available);
                    score.replay_location = if s.replay_available == "1" {
                        use engine::beatmaps::osu::ReplayDownloader;
                        ReplayLocation::Online(Arc::new(ReplayDownloader::new(score.score.clone(), id.parse().unwrap_or_default())))
                    } else {
                        ReplayLocation::OnlineNotExist
                    };

                    score
                }).collect())

            } else {
                Err(tataku::Error::String("no osu map".to_owned()))
            }
        }
    }

}

mod quaver {
    use super::*;
    use engine::beatmaps::quaver::QuaverReplayDownloader;

    pub async fn fetch_beatmap_id(map_hash: &String) -> Option<u32> {
        let url = format!("https://api.quavergame.com/v1/maps/{map_hash}");
        let resp = make_request::<QuaverResponse>(&url).ok()?;

        resp.map.map(|m|m.id)
    }


    pub async fn get_scores(
        map_hash: Md5Hash,
        infos: &GamemodeInfos,
    ) -> Vec<IngameScore> {
        let map_hash = map_hash.to_string();
        match get_scores_internal(&map_hash, infos).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting quaver scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(
        map_hash: &String,
        infos: &GamemodeInfos,
    ) -> engine::tataku::Result<Vec<IngameScore>> {
        let info = infos.get_info("mania")?;
        let ok_mods = Mods::mods_for_playmode_as_hashmap(info);

        // need to fetch the beatmap id, because peppy doesnt allow getting scores by hash :/
        let Some(id) = fetch_beatmap_id(map_hash).await else {return Err(engine::tataku::Error::String("no osu map".to_owned()))};
        let url = format!("https://api.quavergame.com/v1/scores/map/{id}");

        let resp = make_request::<QuaverResponse>(&url)?;

        Ok(resp.scores.unwrap_or_default().iter().map(|s| {
            let mut judgments = HashMap::new();
            judgments.insert("x50".to_owned(),   s.count_okay as u16);
            judgments.insert("x100".to_owned(),  s.count_good as u16);
            judgments.insert("x300".to_owned(),  s.count_marv as u16);
            judgments.insert("xgeki".to_owned(), s.count_perf as u16);
            judgments.insert("xkatu".to_owned(), s.count_great as u16);
            judgments.insert("xmiss".to_owned(), s.count_miss as u16);


            let mut score = Score {
                username: s.user.username.clone(),
                score: s.total_score,
                combo: s.max_combo as u16,
                max_combo: s.max_combo as u16,
                judgments,
                speed: GameSpeed::default(),
                accuracy: s.accuracy / 100.0,
                ..Score::default()
            };

            // check mods
            for m in s.mods_string.split(", ") {
                if m.ends_with("x")
                && let Ok(speed) = m.trim_end_matches("x").parse() {
                    score.speed = GameSpeed::from_f32(speed);
                    continue;
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
    use super::*;
    use engine::beatmaps::TatakuReplayDownloader;

    #[derive(Serialize, Deserialize)]
    struct TatakuScore {
        score_id: u64,
        score_hash: Option<String>,
        score: Score
    }

    pub async fn get_scores(
        map_hash: &str,
        playmode: &str,
        settings: &engine::Settings
    ) -> Vec<IngameScore> {
        match get_scores_internal(map_hash, playmode, settings).await {
            Ok(maps) => maps,
            Err(e) => {
                warn!("error getting tataku scores: {e}");
                Vec::new()
            }
        }
    }

    async fn get_scores_internal(
        map_hash: &str,
        playmode: &str,
        settings: &engine::Settings
    ) -> engine::tataku::Result<Vec<IngameScore>> {
        let base = settings.connection().score_url.clone();
        let url = format!("{base}/api/get_scores?hash={map_hash}&mode={playmode}");

        let maps = make_request::<Vec<TatakuScore>>(&url)?;

        Ok(maps.into_iter().map(|s| {
            let mut score = IngameScore::new(s.score, false, false);
            if s.score_hash.is_none() {
                score.replay_location = ReplayLocation::OnlineNotExist;
            } else {
                score.replay_location = ReplayLocation::Online(Arc::new(
                    TatakuReplayDownloader::new(s.score_id, s.score_hash)
                ));
            }
            score
        }).collect())
    }

}


fn make_request<T: serde::de::DeserializeOwned>(url: &str) -> engine::tataku::Result<T> {
    let a = ureq::get(url)
        .call()?
        .into_body()
        .read_to_vec()?;

    Ok(serde_json::from_slice(&a)?)
}
