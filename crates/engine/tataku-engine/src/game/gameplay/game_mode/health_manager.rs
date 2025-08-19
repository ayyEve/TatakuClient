use crate::prelude::*;

pub trait HealthManager: Send + Sync {
    /// is this health manager out of health (did the user fail?)
    fn is_dead(&self, song_over: bool) -> bool;

    /// ratio of health to max health
    fn get_ratio(&self) -> f32;

    /// reset the health to its default
    /// ie, the user restarted the map
    fn reset(&mut self);

    /// verify health is within valid bounds
    fn validate_health(&mut self) {}

    /// apply a hit judgment to ourself
    fn apply_hit(&mut self, hit_judgment: &HitJudgment, score: &IngameScore);
}

#[derive(Default2)]
pub struct DefaultHealthManager {
    #[default(80.0)] current_health: f32,
    #[default(80.0)] initial_health: f32,
    #[default(80.0)] max_health: f32,
}
impl HealthManager for DefaultHealthManager {
    fn is_dead(&self, _song_over: bool) -> bool {
        self.current_health <= 0.0
    }

    fn get_ratio(&self) -> f32 {
        self.current_health / self.max_health
    }

    fn reset(&mut self) {
        self.current_health = self.initial_health;
    }

    fn validate_health(&mut self) {
        if self.current_health < 0.0 { self.current_health = 0.0 }
        if self.current_health > self.max_health { self.current_health = self.max_health }
    }

    fn apply_hit(&mut self, hit_judgment: &HitJudgment, _score: &IngameScore) {
        self.current_health += hit_judgment.health;
        self.validate_health();
    }
}
