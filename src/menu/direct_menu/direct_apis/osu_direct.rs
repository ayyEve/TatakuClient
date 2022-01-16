use super::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};


pub struct OsuDirect {}
impl OsuDirect {
    pub fn new() -> Self {Self{}}
}
impl DirectApi for OsuDirect {
    fn api_name(&self) -> &'static str {"Osu"}
    fn supported_modes(&self) -> Vec<PlayMode> {
        vec![
            PlayMode::Standard,
            PlayMode::Taiko,
            PlayMode::Catch,
            PlayMode::Mania
        ]
    }

    fn do_search(&mut self, search_params:SearchParams) -> Vec<Arc<dyn DirectDownloadable>> {
        println!("[OsuDirect] searching");
        let settings = Settings::get();


        // TODO: do a proper sort (and convert from generic sort to osu sort number)
        let sort = search_params.sort.unwrap_or_default() as u8;
        let status:OsuMapStatus = search_params.map_status.unwrap_or_default().into();

        
        // url = "https://osu.ppy.sh/web/osu-search.php?u=[]&h=[]".to_owned();
        let url = format!(
            "https://osu.ppy.sh/web/osu-search.php?u={}&h={:x}&m={}&p={}&s={}&r={}{}",
            /*   username  */ settings.osu_username,
            /*   password  */ md5::compute(settings.osu_password),
            /*   playmode  */ search_params.mode.unwrap_or_default() as u8,
            /*   page num  */ search_params.page,
            /*   sort num  */ sort,
            /*  rank state */ status as i8,
            /* text search */ if let Some(t) = search_params.text {format!("&q={}", t)} else {String::new()}
        );

        let body = reqwest::blocking::get(url)
            .expect("error with request")
            .text()
            .expect("error converting to text");

        let mut lines = body.split('\n');
        let count = lines.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
        println!("[OsuDirect] got {} items", count);

        // parse items into list, and return list
        let mut items = Vec::new();
        for line in lines {
            if line.len() < 5 {continue}
            // why does this work
            items.push(Arc::new(OsuDirectDownloadable::from_str(line)) as Arc<dyn DirectDownloadable>)
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

    downloading: Arc<AtomicBool>
}
impl OsuDirectDownloadable {
    pub fn from_str(str:&str) -> Self {
        // println!("reading {}", str);
        let mut split = str.split('|');

        // 867737.osz|The Quick Brown Fox|The Big Black|Mismagius|1|9.37143|2021-06-25T02:25:11+00:00|867737|820065|||0||Easy ★1.9@0,Normal ★2.5@0,Advanced ★3.2@0,Hard ★3.6@0,Insane ★4.8@0,Extra ★5.6@0,Extreme ★6.6@0,Remastered Extreme ★6.9@0,Riddle me this riddle me that... ★7.5@0
        // filename, artist, title, creator, ranking_status, rating, last_update, beatmapset_id, thread_id, video, storyboard, filesize, filesize_novideo||filesize, difficulty_names

        let filename = split.next().expect("[OsuDirect] err:filename").to_owned();
        let artist = split.next().expect("[OsuDirect] err:artist").to_owned();
        let title = split.next().expect("[OsuDirect] err:title").to_owned();
        let creator = split.next().expect("[OsuDirect] err:creator").to_owned();
        let _ranking_status = split.next();
        let _rating = split.next();
        let _last_update = split.next();
        let set_id = split.next().expect("set_id").to_owned();

        Self {
            set_id,
            filename,
            artist,
            title,
            creator,

            downloading: Arc::new(AtomicBool::new(false))
        }
    }
}
impl DirectDownloadable for OsuDirectDownloadable {
    fn download(&self) {
        if self.is_downloading() {return}

        self.downloading.store(true, SeqCst);

        let download_dir = format!("downloads/{}", self.filename);
        let settings = Settings::get();
        
        let username = settings.osu_username;
        let password = settings.osu_password;
        let url = format!("https://osu.ppy.sh/d/{}?u={}&h={:x}", self.filename, username, md5::compute(password));

        perform_download(url, download_dir);
    }

    fn audio_preview(&self) -> Option<String> {
        // https://b.ppy.sh/preview/100.mp3
        Some(format!("https://b.ppy.sh/preview/{}.mp3", self.set_id))
    }


    fn filename(&self) -> String {self.filename.clone()}
    fn title(&self) -> String {self.title.clone()}
    fn artist(&self) -> String {self.artist.clone()}
    fn creator(&self) -> String {self.creator.clone()}
    fn get_download_progress(&self) -> f32 {0.0}
    fn is_downloading(&self) -> bool {self.downloading.load(SeqCst)}
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
