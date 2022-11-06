use crate::prelude::*;


/// used for ingame_manager leaderboard
#[derive(Clone)]
pub struct IngameScore {
    pub score: Score,
    /// is this the current score
    pub is_current: bool,
    /// is this a user's previous score?
    pub is_previous: bool,

    /// is this score from the internet? (ie not local)
    pub replay_location: ReplayLocation,

    // snapshots
    /// time, acc
    pub accs: Vec<(f32, f32)>,
    /// time, health
    pub healths: Vec<(f32, f32)>,
    /// time, perf
    pub perfs: Vec<(f32, f32)>,
}
impl IngameScore {
    pub fn new(score: Score, is_current: bool, is_previous: bool) -> Self {
        Self {
            score, 
            is_current,
            is_previous,
            replay_location: ReplayLocation::Local,

            accs: Vec::new(),
            healths: Vec::new(),
            perfs: Vec::new()
        }
    }

    pub async fn get_replay(&self) -> Option<Replay> {
        info!("downloading: {:#?}", self.replay_location);

        match &self.replay_location {
            ReplayLocation::Local => {
                match crate::databases::get_local_replay_for_score(&self.score) {
                    Ok(replay) => return Some(replay),
                    Err(e) => NotificationManager::add_error_notification("Error loading replay", e).await,
                }
            }
            ReplayLocation::Online(downloader) => {
                match downloader.get_replay().await {
                    Ok(replay) => return Some(replay),
                    Err(e) => NotificationManager::add_error_notification("Error reading replay", e).await,
                }
            }
            ReplayLocation::OnlineNotExist => {
                // TODO: replay button should be hidden in this case, but im bad coder
                NotificationManager::add_text_notification("Replay is not available :c", 5000.0, Color::RED).await
            }
        }

        None
    }

    pub fn take_snapshot(&mut self, time: f32, health: f32) {
        self.accs.push((time, self.score.accuracy as f32));
        self.healths.push((time, health));
        self.perfs.push((time, self.score.performance));
    }
}

#[derive(Clone, Debug)]
pub enum ReplayLocation {
    Local,
    // url, extention
    Online(Arc<dyn ReplayDownloader>),
    OnlineNotExist,
}



impl core::ops::Deref for IngameScore {
    type Target = Score;

    fn deref(&self) -> &Self::Target {
        &self.score
    }
}
impl core::ops::DerefMut for IngameScore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.score
    }
}
