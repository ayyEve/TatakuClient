use crate::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};


pub struct OsuDirect;

#[async_trait]
impl DirectApi for OsuDirect {
    fn api_name(&self) -> &'static str {"Osu"}
    fn supported_modes(&self) -> Vec<String> {
        vec![
            "osu".to_owned(),
            "taiko".to_owned(),
            "catch".to_owned(),
            "mania".to_owned()
        ]
    }

    async fn do_search(&mut self, search_params:SearchParams) -> Vec<Arc<dyn DirectDownloadable>> {
        trace!("Searching");
        let settings = Settings::get();


        // TODO: do a proper sort (and convert from generic sort to osu sort number)
        let sort = search_params.sort.unwrap_or_default() as u8;
        let status:OsuMapStatus = search_params.map_status.unwrap_or_default().into();

        
        // url = "https://osu.ppy.sh/web/osu-search.php?u=[]&h=[]".to_owned();
        let url = format!(
            "https://osu.ppy.sh/web/osu-search.php?u={}&h={}&m={}&p={}&s={}&r={}{}",
            /*   username  */ settings.osu_username,
            /*   password  */ settings.osu_password,
            /*   playmode  */ playmode_to_u8(search_params.mode.unwrap_or("osu".to_owned())),
            /*   page num  */ search_params.page,
            /*   sort num  */ sort,
            /*  rank state */ status as i8,
            /* text search */ if let Some(t) = search_params.text {format!("&q={}", t)} else {String::new()}
        );

        let body = reqwest::get(url).await
            .expect("error with request")
            .text().await
            .expect("error converting to text");

        let mut lines = body.split('\n');
        let count = lines.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
        trace!("Got {} items", count);

        // parse items into list, and return list
        let mut items = Vec::new();
        for line in lines {
            if line.len() < 5 {continue}
            if let Some(dl) = OsuDirectDownloadable::from_str(line) {
                // why does this work
                items.push(Arc::new(dl) as Arc<dyn DirectDownloadable>)
            }
        }

        items
    }
}


// TODO: figure out how to get the progress of the download
#[derive(Clone)]
pub struct OsuDirectDownloadable {
    set_id: String,
    filename: String,
    artist: String,
    title: String,
    creator: String,

    progress: Arc<RwLock<DownloadProgress>>,
    downloading: Arc<AtomicBool>,
}
impl OsuDirectDownloadable {
    pub fn from_str(str:&str) -> Option<Self> {
        // trace!("reading {}", str);
        let mut split = str.split('|');

        // 867737.osz|The Quick Brown Fox|The Big Black|Mismagius|1|9.37143|2021-06-25T02:25:11+00:00|867737|820065|||0||Easy ★1.9@0,Normal ★2.5@0,Advanced ★3.2@0,Hard ★3.6@0,Insane ★4.8@0,Extra ★5.6@0,Extreme ★6.6@0,Remastered Extreme ★6.9@0,Riddle me this riddle me that... ★7.5@0
        // filename, artist, title, creator, ranking_status, rating, last_update, beatmapset_id, thread_id, video, storyboard, filesize, filesize_novideo||filesize, difficulty_names

        macro_rules! next {
            () => {
                if let Some(v) = split.next() {
                    v.to_owned()
                } else {
                    return None
                }
            }
        }

        let filename = next!();
        let artist = next!();
        let title = next!();
        let creator = next!();
        let _ranking_status = next!();
        let _rating = next!();
        let _last_update = next!();
        let set_id = next!();

        Some(Self {
            set_id,
            filename,
            artist,
            title,
            creator,
            
            progress: Default::default(),
            downloading: Arc::new(AtomicBool::new(false))
        })
    }
}
impl DirectDownloadable for OsuDirectDownloadable {
    fn download(&self) {
        if self.is_downloading() { return }

        self.downloading.store(true, SeqCst);

        let download_dir = format!("downloads/{}", self.filename);
        let settings = Settings::get();
        
        let username = &settings.osu_username;
        let password = &settings.osu_password;
        let url = format!("https://osu.ppy.sh/d/{}?u={}&h={}", self.filename, username, password);

        perform_download(url, download_dir, self.progress.clone());
    }

    fn audio_preview(&self) -> Option<String> {
        // https://b.ppy.sh/preview/100.mp3
        Some(format!("https://b.ppy.sh/preview/{}.mp3", self.set_id))
    }


    fn filename(&self) -> String { self.filename.clone() }
    fn title(&self) -> String { self.title.clone() }
    fn artist(&self) -> String { self.artist.clone() }
    fn creator(&self) -> String { self.creator.clone() }
    fn get_download_progress(&self) -> &Arc<RwLock<DownloadProgress>> { &self.progress }
    fn is_downloading(&self) -> bool { self.downloading.load(SeqCst) }
}


#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OsuMapStatus {
    Ranked = 1,
    Pending = 2,
    // 3 is ??
    All = 4,
    Graveyarded = 5,
    Approved = 6,
    Loved = 8,
}
impl Into<OsuMapStatus> for MapStatus {
    // pain
    fn into(self) -> OsuMapStatus {
        match self {
            MapStatus::All => OsuMapStatus::All,
            MapStatus::Ranked => OsuMapStatus::Ranked,
            MapStatus::Pending => OsuMapStatus::Pending,
            MapStatus::Graveyarded => OsuMapStatus::Graveyarded,
            MapStatus::Approved => OsuMapStatus::Approved,
            MapStatus::Loved => OsuMapStatus::Loved,
        }
    }
}



pub fn playmode_to_u8(s:String) -> u8 {
    match &*s {
        "osu" => 0,
        "taiko" => 1,
        "catch" => 2,
        "mania" => 3,

        _ => 255
    }
}
