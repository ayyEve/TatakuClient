#![allow(unused, reason = "osu api fields")]
use std::str::FromStr;

use crate::*;
use online_content::{
    *,
    osu_connection::OsuConnection,
};
use tokio::sync::Mutex;

// https://github.com/ppy/osu-web/blob/58514a67d1f38e9842045615993252a8810fd50b/app/Libraries/Search/BeatmapsetSearchRequestParams.php

pub struct OsuDirect {
    capabilities: OnlineContentCapabilities,
    connection: Arc<Mutex<Option<OsuConnection>>>,
}
impl OsuDirect {
    pub fn new(settings: &Settings) -> Self {
        let connection = OsuConnection::setup(settings);

        Self {
            capabilities: OnlineContentCapabilities { 
                engine_id: "osu_direct".to_string(), 
                display_name: "osu!".to_string(), 
                available_types: vec![
                    OnlineContentType::Maps,
                ], 
                search_options: [
                    (
                        "mode".to_owned(),
                        SearchOption::new(
                            "Playmode",
                            vec![
                                OnlineContentSearchData::new("Unset", ""),
                                OnlineContentSearchData::new("Osu", "0"),
                                OnlineContentSearchData::new("Taiko", "1"),
                                OnlineContentSearchData::new("Catch", "2"),
                                OnlineContentSearchData::new("Mania", "3"),
                            ]
                        )
                    ),
                    (
                        "status".to_owned(),
                        SearchOption::new(
                            "Status",
                            vec![
                                OnlineContentSearchData::new("Unset", ""),
                                OnlineContentSearchData::new("Ranked", "1"),
                                OnlineContentSearchData::new("Pending", "2"),
                                OnlineContentSearchData::new("All", "4"),
                                OnlineContentSearchData::new("Graveyarded", "5"),
                                OnlineContentSearchData::new("Approved", "6"),
                                OnlineContentSearchData::new("Loved", "8"),
                            ]
                        )
                    ),
                    (
                        "nsfw".to_owned(),
                        SearchOption::new(
                            "Nsfw",
                            vec![
                                OnlineContentSearchData::new("Yes", "true"),
                                OnlineContentSearchData::new("No", "false"),
                            ]
                        ),
                    ),
                ].into_iter().collect()
            },
            connection: Arc::new(Mutex::new(connection)),
        }
    }
}

impl OnlineContentEngine for OsuDirect {
    fn capabilities(&self) -> &OnlineContentCapabilities { &self.capabilities }

    fn search(
        &self, 
        settings: &Settings, 
        search: OnlineContentSearch,
    ) -> io::AsyncLoader<OnlineContentSearchResults> {
        debug!("Searching: {search:?}");
        let creds = settings.integrations.osu.clone();

        let mode = search.search_values
            .get("mode")
            .map(|s| &**s)
            .unwrap_or_default();

        let sort = search.search_values
            .get("sort")
            .map(|s| &**s)
            .unwrap_or_default();

        let status = search.search_values
            .get("status")
            .map(|s| &**s)
            .unwrap_or_default();

        let nsfw = search.search_values
            .get("nsfw")
            .map(|s| &**s)
            .unwrap_or_default();

        let query = search.query.unwrap_or_default();
        let page = search.page;
        let genre = "";
        let language = "";
        
        let url = format!(
            "https://osu.ppy.sh/api/v2/beatmapsets/search?g={genre}&l={language}&m={mode}&nsfw={nsfw}&q={query}&sort={sort}&s={status}&page={page}",
        );

        let connection = self.connection.clone();

        io::AsyncLoader::new(async move {
            debug!("Osu!direct: {url}");
            
            let mut connection = connection
                .lock()
                .await;

            let mut request = ureq::get(url);

            if let Some(conn) = &*connection {
                let bearer = conn.bearer();
                // request = request.bearer_auth(conn.auth());
                request = request.header("Authorization", bearer);
            }


            let response = request
                .call()
                .expect("Error with request");

            let body = response
                .into_body()
                .read_to_string()
                .expect("Error converting to text");

            // debug!("Got results: \n{body}");
            // std::fs::write("/tmp/a.json", &body).unwrap();

            let results = 
                serde_json::from_str::<OsuSearchResults>(&body)
                .inspect_err(|e| error!("Error parsing osu!direct response: {e:?}"))
                .unwrap_or_default();

            let count = results.total as usize;

            // parse items into list, and return list
            let items = results
                .beatmapsets
                .into_iter()
                .enumerate()
                .map(|(id, i)| {
                    let filename = format!("{}.osz", i.id);
                    let url = format!(
                        "https://osu.ppy.sh/d/{filename}?u={}&h={}",
                        creds.username,
                        creds.password,
                    );

                    OnlineContentItem { 
                        id, 
                        display: format!("{}// {} - {}", i.creator, i.artist, i.title), 

                        item_type: OnlineContentItemType::Map { 
                            artist: i.artist, 
                            title: i.title, 
                            creator: i.creator, 
                            map_hashes: i.beatmaps
                                .iter()
                                .filter_map(|i| common::Md5Hash::from_str(&i.checksum).ok())
                                .collect(),
                        },
                        download: Downloadable::new(
                            format!("downloads/{filename}"),
                            move || io::Downloader::download_url(url.clone(), 5),
                        ), 
                        audio_preview: Some(format!("https:{}", i.preview_url))
                    }

                })
                .collect();

            OnlineContentSearchResults {
                items,
                count,
            }
        })
    }
}


#[derive(Default)]
#[derive(Deserialize)]
struct OsuSearchResults {
    error: Option<String>,
    total: u64,
    cursor_string: String,
    recommended_difficulty: Option<f32>,

    // search: ResultsSearch,
    // cursor: ResultsCursor,
    beatmapsets: Vec<ResultsBeatmapSet>,
}

// #[derive(Default)]
// #[derive(Deserialize)]
// struct ResultsCursor {
//     approved_date: u64,
//     id: u32,
// }

// #[derive(Default)]
// #[derive(Deserialize)]
// struct ResultsSearch {
//     sort: String,
// }



#[derive(Deserialize)]
struct ResultsBeatmapSet {
    id: u64,

    artist: String,
    // artist_unicode: String,
    title: String,
    // title_unicode: String,
    creator: String,
    // user_id: u64,
    
    // rating: Option<f32>,

    // favourite_count: u32,
    // genre_id: u8,
    // language_id: u8,
    // track_id: Option<u64>,

    // hype: Option<>,
    // covers: ResultsCovers,

    // nsfw: bool,
    // offset: Option<f32>,
    // play_count: u64,
    preview_url: String,
    // source: String,
    // spotlight: bool,
    // status: String,
    // video: bool,

    // bpm: f32,
    // can_be_hyped: bool,
    // deleted_at: Option<String>

    // .. blah

    // ranked: u8,
    // ranked_date: Option<String>,

    // tags: String,

    beatmaps: Vec<ResultsBeatmap>
}

#[derive(Deserialize)]
struct ResultsBeatmap {
    id: u64,
    // blah...
    checksum: String,
}



// #[tokio::test]
// async fn test() {
//     let settings = Settings::load_from("game/settings.json");
//     let search = OnlineContentSearch {
//         engine_id: "osu".into(),
//         search_type: vec![OnlineContentType::Maps],
//         page: 0,
//         search_values: vec![
//             OnlineContentSearchValue::new("mode", "1"),
//         ].into(),
//         query: None,
//     };

//     let a =  OsuDirect::new(&settings);
//     let results = a.search(
//         &settings,
//         search
//     );

//     while !results.is_complete() {
//         tokio::task::yield_now().await;
//     }

//     let results = results.check().unwrap();
//     println!("results: {results:?}");

// }
