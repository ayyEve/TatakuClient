use crate::*;
use common::reflect::*;
use common::Score;

/// used for ingame_manager leaderboard
#[derive(Reflect)]
#[derive(Clone, Debug, Default)]
pub struct IngameScore {
    /// internal id used for score lists
    pub id: usize,

    // TODO: short mods list string

    #[reflect(flatten)]
    pub score: Score,

    pub health: f32,

    pub score_type: ScoreType,

    /// is this score from the internet? (ie not local)
    #[reflect(skip)]
    pub replay_location: ReplayLocation,
}
impl IngameScore {
    pub fn new(score: Score, is_current: bool, is_previous: bool) -> Self {
        Self {
            id: 0,
            score, 
            health: 1.0,
            score_type: ScoreType::new(is_current, is_previous),
            replay_location: ReplayLocation::Local,
        }
    }

    pub fn insert_stat(&mut self, stat: gameplay::stats::GameModeStat, value: f32) {
        let key = stat.name.to_owned();

        if let Some(values) = self.score.stat_data.get_mut(&key) {
            values.push(value);
        } else {
            self.score.stat_data.insert(key, vec![value]);
        }
    }

    /// group the data into sets of groups
    /// the hashmap is indexed by the group name, and the data is a hashmap of stat name, and values for said stat
    /// note that this will not include stats that dont have at least one value
    pub fn stats_into_groups(&self, groups: &Vec<gameplay::stats::StatGroup>) -> HashMap<String, HashMap<String, Vec<f32>>> {
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

#[derive(Clone, Debug, Default)]
pub enum ReplayLocation {
    #[default]
    Local,
    // url, extention
    Online(Arc<dyn beatmaps::ReplayDownloader>),
    OnlineNotExist,
}

#[derive(Reflect)]
#[derive(Copy, Clone, Debug, Default)]
pub enum ScoreType {
    #[default] Default,

    /// Is this the current score?
    Current,
    
    /// Is this a user's previous score?
    Previous,
}
impl ScoreType {
    fn new(is_current: bool, is_previous: bool) -> Self {
        if is_current { 
            Self::Current
        } else if is_previous {
            Self::Previous
        } else {
            Self::Default
        }
    }
}