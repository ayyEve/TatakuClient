#![allow(unused, dead_code)]
use super::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

// request: https://github.com/Quaver/Quaver/blob/ui-redesign/Quaver.Shared/Online/API/MapsetSearch/APIRequestMapsetSearch.cs

pub struct QuaverDirect {}
impl QuaverDirect {
    pub fn new() -> Self {Self{}}
}
#[async_trait]
impl DirectApi for QuaverDirect {
    fn api_name(&self) -> &'static str {"Quaver"}
    fn supported_modes(&self) -> Vec<PlayMode> {vec!["mania".to_owned()]}

    async fn do_search(&mut self, search_params:SearchParams) -> Vec<Arc<dyn DirectDownloadable>> {
        trace!("Searching");

        let mut params = Vec::new();
        macro_rules! check_val {
            ($val: expr, $key:expr) => {
                if let Some(val) = $val {
                    params.push(format!("{}={}", $key, val))
                }
            }
        }
        check_val!(search_params.min_diff, "mindiff");
        check_val!(search_params.max_diff, "maxdiff");
        check_val!(search_params.min_length, "minlength");
        check_val!(search_params.max_length, "maxlength");
        check_val!(search_params.min_lns, "minlns");
        check_val!(search_params.max_lns, "maxlns");
        check_val!(search_params.min_combo, "mincombo");
        check_val!(search_params.max_combo, "maxcombo");
        
        // https://api.quavergame.com/v1/mapsets/maps/search
        let url = format!(
            "https://api.quavergame.com/v1/mapsets/maps/search?page={}{}{}",
            search_params.page,
            if params.len() > 0 {"&"} else {""},
            params.join("&")
        );

        let body = reqwest::blocking::get(url)
            .expect("Error with request")
            .text()
            .expect("Error converting to text");


        let deserialized:QuaverMapsetRequest = serde_json::from_str(&body).expect("Error deserializing response");

        let mut items = Vec::new();
        for i in deserialized.mapsets {
            let i = Arc::new(QuaverDirectDownloadable::new(i)) as Arc<dyn DirectDownloadable>;
            items.push(i)
        }

        items
    }
}


// TODO: figure out how to get the progress of the download
#[derive(Clone)]
pub struct QuaverDirectDownloadable {
    // easier to just store the whole item tbh
    item: QuaverMapset,

    filename: String,

    // keys: 

    downloading: Arc<AtomicBool>
}
impl QuaverDirectDownloadable {
    fn new(item: QuaverMapset) -> Self {
        let filename = format!("{}.qp", item.id);
        Self {
            item,
            filename,
            downloading: Arc::new(AtomicBool::new(false))
        }
    }
}
#[async_trait]
impl DirectDownloadable for QuaverDirectDownloadable {
    async fn download(&self) {
        if self.is_downloading() {return}

        self.downloading.store(true, SeqCst);

        let download_dir = format!("downloads/{}", self.filename);
        let settings = get_settings!();
        
        let username = &settings.osu_username;
        let password = &settings.osu_password;
        let url = format!("https://osu.ppy.sh/d/{}?u={}&h={}", self.filename, username, password);

        perform_download(url, download_dir);
    }

    fn audio_preview(&self) -> Option<String> {
        // https://b.ppy.sh/preview/100.mp3
        Some(format!("https://cdn.quavergame.com/audio-previews/{}.mp3", self.item.id))
    }


    fn filename(&self) -> String {self.filename.clone()}
    fn title(&self) -> String {self.item.title.clone()}
    fn artist(&self) -> String {self.item.artist.clone()}
    fn creator(&self) -> String {self.item.creator_username.clone()}
    fn get_download_progress(&self) -> f32 {0.0}
    fn is_downloading(&self) -> bool {self.downloading.load(SeqCst)}
}


// deserialization structs
#[derive(Deserialize)]
struct QuaverMapsetRequest {
    status: u16,
    mapsets: Vec<QuaverMapset>
}

#[derive(Deserialize, Clone)]
struct QuaverMapset {
    id: u32,
    creator_id: u32,
    creator_username: String,
    artist: String,
    title: String,
    source: Option<String>,
    tags: String,
    description: Option<String>,
    /// QuaverRankedStatus
    ranked_status: i8,
    date_submitted: String,
    date_last_updated: String,
    bpms: Vec<f32>,
    /// @QuaverGameMode
    game_modes: Vec<u8>,

    difficulty_names: Vec<String>,
    difficulty_range: Vec<f32>,

    
    min_length_seconds: f32,
    max_length_seconds: f32,
    min_ln_percent: f32,
    max_ln_percent: f32,
    min_play_count: u32,
    max_play_count: u32,
    min_date_submitted: String,
    max_date_submitted: String,
    min_date_last_updated: String,
    max_date_last_updated: String,
    min_combo: u16,
    max_combo: u16,
}



// https://github.com/Quaver/Quaver.API/blob/551060395225dd66ff3bf702df165ac056e45196/Quaver.API/Enums/RankedStatus.cs
enum QuaverRankedStatus {
    Unsubmitted,
    Unranked,
    Ranked,
    // idk what this is so im ignoring it
    // DanCourse
    Other
}
impl Into<QuaverRankedStatus> for MapStatus {
    // pain
    fn into(self) -> QuaverRankedStatus {
        match self {
            MapStatus::Pending
            | MapStatus::Graveyarded 
                => QuaverRankedStatus::Unranked,

                // not sure how to handle this, maybe just not include it in the query?
            MapStatus::All
            | MapStatus::Ranked 
            | MapStatus::Approved 
            | MapStatus::Loved 
                => QuaverRankedStatus::Ranked,
        }
    }
}


/// key count helper
/// called game mode in quaver
enum QuaverGameMode {
    FourKey,
    SevenKey
}