use crate::prelude::*;

#[derive(Debug)]
pub struct QuaverReplayDownloader(Score, u32);

impl QuaverReplayDownloader {
    pub fn new(score: Score, id: u32) -> Self {
        Self(score, id)
    }
}


#[async_trait]
impl ReplayDownloader for QuaverReplayDownloader {
    async fn get_replay(&self) -> TatakuResult<Replay> {
        Err(TatakuError::String("Not Implemented".to_owned()))
        // https://quavergame.com/download/replay/48727123

        // let url = format!("https://quavergame.com/download/replay/{}", self.1);

        // // this should be a .qr file, but cloudflare is breaking everything right now
        // let bytes = reqwest::get(url).await?.bytes().await?;
        
        // // check if the received data 
        // if bytes.len() == 0 {
        //     return Err(TatakuError::String("Downloaded file was empty".to_owned()));
        // }

        // // TODO! parse replay
        // Ok(replay)
    }
}