use crate::*;
use common::Score;
use beatmaps::osu::osu_replay_converter::OsuReplay;

#[derive(Debug)]
pub struct OsuReplayDownloader(Score, u64);

impl OsuReplayDownloader {
    pub fn new(score: Score, score_id: u64) -> Self {
        Self(score, score_id)
    }
}

impl beatmaps::ReplayDownloader for OsuReplayDownloader {
    fn get_replay(&self, settings: &Settings) -> tataku::Result<Score> {
        let key = settings.integrations.osu.api_key.clone();

        let url = format!("https://osu.ppy.sh//api/get_replay?k={key}&s={}", self.1);

        // what gets downloaded from the api is not the full .osr file, its just the lzma stream.
        let bytes = ureq::get(url)
            .call()?
            .into_body()
            .read_to_vec()?;
    
        // check if the received data 
        if bytes.is_empty() {
            return Err(tataku::Error::String("Downloaded file was empty".to_owned()));
        }


        // peppy is cancer, file is a json of content = base64(replay)
        #[derive(Deserialize)]
        struct Wrapper { content: Option<String>, error: Option<String> }

        let data:Wrapper = serde_json::from_slice(&bytes)?;

        if let Some(content) = &data.content {
            let data = tataku::Cryptography::decode_base64(content)
                .map_err(|e| tataku::Error::String(format!("error decoding osu replay: {e}")))?;
            Ok(OsuReplay::replay_from_score_and_lzma(&self.0, &mut data.as_ref())?)
        } else {
            Err(tataku::Error::String(data.error.unwrap_or("peppy api sucks".to_owned())))
        }
    }
}
