mod osu_replay;
mod quaver_replay;
mod tataku_replay;

pub use osu_replay::*;
pub use quaver_replay::*;
pub use tataku_replay::*;


use crate::prelude::*;

#[async_trait]
pub trait ReplayDownloader: Send + Sync + std::fmt::Debug {
    async fn get_replay(&self) -> TatakuResult<Replay>;
}