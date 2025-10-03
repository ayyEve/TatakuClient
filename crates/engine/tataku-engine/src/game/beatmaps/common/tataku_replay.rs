use crate::*;
use common::Score;
use common::serialization::*;

pub trait ReplayDownloader: Send + Sync + std::fmt::Debug {
    fn get_replay(&self, settings: &Settings) -> tataku::Result<Score>;
}

#[derive(Debug)]
pub struct TatakuReplayDownloader(u64, Option<String>);
impl TatakuReplayDownloader {
    pub fn new(id: u64, hash: Option<String>) -> Self {
        Self(id, hash)
    }
}
impl ReplayDownloader for TatakuReplayDownloader {
    fn get_replay(&self, settings: &Settings) -> tataku::Result<Score> {
        let base = settings.connection().score_url.clone();

        let url = if let Some(hash) = &self.1 {
            format!("{base}/replay_file?hash={hash}")
        } else {
            format!("{base}/replay_file?score_id={}", self.0)
        };
        
        // this will be a full .ttkr file, aka a replay binary file
        let bytes = reqwest::blocking::get(url)?.error_for_status()?.bytes()?;
        
        // check if the received data 
        if bytes.is_empty() {
            return Err(tataku::Error::String("Downloaded file was empty".to_owned()));
        }
        
        let score = Score::read(&mut SerializationReader::new(bytes.to_vec()))?;
        Ok(score)
    }
}
