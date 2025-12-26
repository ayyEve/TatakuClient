use crate::*;
use gameplay::{
    IngameScore,
    judgments::HitJudgment,
};

pub trait Health: Send + Sync {
    /// did the user fail?
    fn is_dead(&self, song_over: bool) -> bool;

    /// ratio of health to max health
    fn get_ratio(&self) -> f32;

    /// reset the health to its default
    /// ie, the user restarted the map
    fn reset(&mut self);

    /// apply a hit judgment to ourself
    fn apply_hit(&mut self, hit_judgment: &HitJudgment, score: &IngameScore);
}

#[derive(Default2)]
pub struct DefaultHealth {
    #[default(80.0)] current_health: f32,
    #[default(80.0)] initial_health: f32,
    #[default(80.0)] max_health: f32,
}
impl Health for DefaultHealth {
    fn is_dead(&self, _song_over: bool) -> bool {
        self.current_health <= 0.0
    }

    fn get_ratio(&self) -> f32 {
        self.current_health / self.max_health
    }

    fn reset(&mut self) {
        self.current_health = self.initial_health;
    }

    fn apply_hit(&mut self, hit_judgment: &HitJudgment, _score: &IngameScore) {
        self.current_health = f32::clamp(
            self.current_health + hit_judgment.health,
            0.0,
            self.max_health
        )
    }
}
