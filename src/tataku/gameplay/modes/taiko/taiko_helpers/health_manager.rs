use crate::prelude::*;

const MAX_HEALTH:f32 = 200.0;
const PASS_HEALTH:f32 = MAX_HEALTH / 2.0;

pub struct TaikoBatteryHealthManager {
    health: f32,

    health_per_300: f32,
    health_per_100: f32,
    health_per_miss: f32,
}
impl TaikoBatteryHealthManager {
    pub fn new(
        health_per_300: f32,
        health_per_100: f32,
        health_per_miss: f32,
    ) -> Self {
        Self {
            health: 0.0,
            health_per_300,
            health_per_100,
            health_per_miss,
        }
    }
}

impl HealthManager for TaikoBatteryHealthManager {
    fn is_dead(&self, song_over: bool) -> bool {
        if !song_over { return false }
        self.health < PASS_HEALTH
    }

    fn get_ratio(&self) -> f32 {
        self.health / MAX_HEALTH
    }

    fn reset(&mut self) {
        self.health = 0.0;
    }

    fn apply_hit(&mut self, hit_judgment: &HitJudgment, _score: &IngameScore) {
        self.health += match hit_judgment.id {
            "x300" => self.health_per_300,
            "x100" => self.health_per_100,
            "xmiss" => self.health_per_miss,
            _ => return
        };

        self.validate_health()
    }

    fn validate_health(&mut self) {
        self.health = self.health.clamp(0.0, MAX_HEALTH);
    }
}