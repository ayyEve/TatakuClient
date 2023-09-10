use crate::prelude::*;


pub struct OsuApi;
impl OsuApi {
    pub async fn get_beatmap_by_hash(hash: impl Display) -> TatakuResult<Option<OsuApiBeatmap>> {
        // let hash = hash.as_ref();
        
        // need to query the osu api to get the set id for this hashmap
        let key = Settings::get().osu_api_key.clone();

        // if no key, return error
        if key.is_empty() { return TatakuResult::Err(TatakuError::String("no osu api key".to_owned())) }

        // do the query
        let api_resp = reqwest::get(format!("https://osu.ppy.sh/api/get_beatmaps?k={key}&h={hash}")).await.map_err(|e|TatakuError::String(format!("error with osu api beatmap request: {e}")))?;
        let data = api_resp.text().await.map_err(|e|TatakuError::String(format!("error getting text for osu api beatmap request: {e}")))?;
        
        // we got a response, return it
        debug!("osu get_beatmaps response: {data}");
        let raw:Vec<RawOsuApiBeatmap> = serde_json::from_str(&data)?;
        let mut raw = raw.into_iter();
        Ok(raw.next().map(|a|a.into()))
    }
}


#[allow(unused)]
#[derive(Deserialize, Debug)]
struct RawOsuApiBeatmap {
    beatmap_id: String,
    beatmapset_id: String,
    artist: String,
    title: String,
    version: String,
    creator: String,
    creator_id: String,
    source: Option<String>,
    mode: String,
    tags: String,
    bpm: String,
    max_combo: String, 

    approved: String, // 4 = loved, 3 = qualified, 2 = approved, 1 = ranked, 0 = pending, -1 = WIP, -2 = graveyard
    submit_date: String,
    approved_date: Option<String>,
    last_update: Option<String>,
    
    diff_aim: Option<String>,
    diff_speed: Option<String>,
    diff_size: Option<String>,
    diff_overall: Option<String>,
    diff_approach: Option<String>,
    diff_drain: Option<String>,
    difficultyrating: String,
    
    hit_length: String,
    total_length: String,
    
    file_md5: String,
    genre_id: Option<String>, // 0 = any, 1 = unspecified, 2 = video game, 3 = anime, 4 = rock, 5 = pop, 6 = other, 7 = novelty, 9 = hip hop, 10 = electronic, 11 = metal, 12 = classical, 13 = folk, 14 = jazz (note that there's no 8)
    language_id: Option<String>, // 0 = any, 1 = unspecified, 2 = english, 3 = japanese, 4 = chinese, 5 = instrumental, 6 = korean, 7 = french, 8 = german, 9 = swedish, 10 = spanish, 11 = italian, 12 = russian, 13 = polish, 14 = other
    count_normal: String,
    count_slider: String,
    count_spinner: String,
    
    rating: String,
    playcount: Option<String>,
    passcount: Option<String>,
    favourite_count: Option<String>, 
    
    storyboard: String, // 0, 1
    video: String, // 0, 1
    download_unavailable: String, // 0, 1
    audio_unavailable: String, // 0, 1
}
impl Into<OsuApiBeatmap> for RawOsuApiBeatmap {
    fn into(self) -> OsuApiBeatmap {
        OsuApiBeatmap {
            beatmap_id: self.beatmap_id.parse().unwrap_or_default(),
            beatmapset_id: self.beatmapset_id.parse().unwrap_or_default(),

        }
    }
}

#[derive(Debug)]
pub struct OsuApiBeatmap {
    pub beatmap_id: u32,
    pub beatmapset_id: u32,
}

#[test]
fn test() {
    let r = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    r.block_on(async {
        Settings::load().await;

        let x = OsuApi::get_beatmap_by_hash("b512dc9b054db498689150556bce5533").await;
        println!("{x:?}")
    });
}