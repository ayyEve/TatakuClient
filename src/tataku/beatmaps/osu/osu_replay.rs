use crate::prelude::*;

#[derive(Debug)]
pub struct OsuReplayDownloader(Score, u64);

impl OsuReplayDownloader {
    pub fn new(score: Score, score_id: u64) -> Self {
        Self(score, score_id)
    }
}


#[async_trait]
impl ReplayDownloader for OsuReplayDownloader {
    async fn get_replay(&self) -> TatakuResult<Replay> {
        let key = Settings::get().osu_api_key.clone();

        let url = format!("https://osu.ppy.sh//api/get_replay?k={key}&s={}", self.1);

        // what gets downloaded from the api is not the full .osr file, its just the lzma stream.
        let bytes = reqwest::get(url).await?.bytes().await?;
    
        // check if the received data 
        if bytes.len() == 0 {
            return Err(TatakuError::String("Downloaded file was empty".to_owned()));
        }


        // peppy is cancer, file is a json of content = base64(replay)
        #[derive(Deserialize)]
        struct Wrapper { content: Option<String>, error: Option<String> }

        let data:Wrapper = serde_json::from_slice(&bytes)?;

        if let Some(content) = &data.content {
            let data = decode_base64(content).map_err(|e| TatakuError::String(format!("error decoding osu replay: {e}")))?;
            Ok(OsuReplay::replay_from_score_and_lzma(&self.0, &mut data.as_ref())?)
        } else {
            Err(TatakuError::String(data.error.unwrap_or("peppy api sucks".to_owned())))
        }
    }
}
