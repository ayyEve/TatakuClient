use crate::prelude::*;


#[async_trait]
pub trait ReplayDownloader: Send + Sync + std::fmt::Debug {
    async fn get_replay(&self) -> TatakuResult<Score>;
}

#[derive(Debug)]
pub struct TatakuReplayDownloader(u64, Option<String>);

impl TatakuReplayDownloader {
    pub fn new(id: u64, hash: Option<String>) -> Self {
        Self(id, hash)
    }
}


#[async_trait]
impl ReplayDownloader for TatakuReplayDownloader {
    async fn get_replay(&self) -> TatakuResult<Score> {
        let base = Settings::get().score_url.clone();

        let url = if let Some(hash) = &self.1 {
            format!("{base}/replay_file?hash={hash}")
        } else {
            format!("{base}/replay_file?score_id={}", self.0)
        };
        

        // this will be a full .ttkr file, aka a replay binary file
        let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
        
        // check if the received data 
        if bytes.len() == 0 {
            return Err(TatakuError::String("Downloaded file was empty".to_owned()));
        }

        let score = Score::read(&mut SerializationReader::new(bytes.to_vec()))?;
        Ok(score)
    }
}