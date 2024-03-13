use crate::prelude::*;

/// used for ingame_manager leaderboard
#[derive(Clone, Debug)]
pub struct IngameScore {
    pub score: Score,
    /// is this the current score
    pub is_current: bool,
    /// is this a user's previous score?
    pub is_previous: bool,

    /// is this score from the internet? (ie not local)
    pub replay_location: ReplayLocation,
}
impl IngameScore {
    pub fn new(score: Score, is_current: bool, is_previous: bool) -> Self {
        Self {
            score, 
            is_current,
            is_previous,
            replay_location: ReplayLocation::Local,
        }
    }

    pub async fn get_replay(&self) -> Option<Replay> {
        info!("downloading: {:#?}", self.replay_location);

        match &self.replay_location {
            ReplayLocation::Local => {
                match get_local_replay_for_score(&self.score) {
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

    pub fn insert_stat(&mut self, stat: GameModeStat, value: f32) {
        let key = stat.name.to_owned();

        if let Some(values) = self.score.stat_data.get_mut(&key) {
            values.push(value)
        } else {
            self.score.stat_data.insert(key, vec![value]);
        }
    }

    /// group the data into sets of groups
    /// the hashmap is indexed by the group name, and the data is a hashmap of stat name, and values for said stat
    /// note that this will not include stats that dont have at least one value
    pub fn stats_into_groups(&self, groups: &Vec<StatGroup>) -> HashMap<String, HashMap<String, Vec<f32>>> {
        let mut output = HashMap::new();

        for group in groups {
            let mut data = HashMap::new();

            for stat in group.stats.iter() {
                if let Some(val) = self.score.stat_data.get(&stat.name()) {
                    data.insert(stat.name(), val.clone());
                }
            }
            output.insert(group.name(), data);
        }

        output
    }

}

impl From<&IngameScore> for CustomElementValue {
    fn from(score: &IngameScore) -> Self {

        // let the score parser handle most of the work
        let score:CustomElementValue = (&score.score).into();

        // TODO: add more things?

        score
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
