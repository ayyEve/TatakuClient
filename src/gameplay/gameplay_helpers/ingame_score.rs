use crate::prelude::*;


/// used for ingame_manager leaderboard
pub struct IngameScore {
    pub score: Score,
    /// is this the current score
    pub is_current: bool,
    /// is this a user's previous score?
    pub is_previous: bool,
}
impl IngameScore {
    pub fn new(score: Score, is_current: bool, is_previous: bool) -> Self {
        Self {
            score, 
            is_current,
            is_previous
        }
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