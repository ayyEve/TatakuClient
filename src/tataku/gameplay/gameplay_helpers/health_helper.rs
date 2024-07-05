use crate::prelude::*;

//TODO: add smoothing (visible_health, lerp on update between current_health and visible_health)
#[derive(Clone)]
pub struct HealthHelper {
    pub current_health: f32,
    pub initial_health: f32,
    pub max_health: f32,

    pub check_fail_at_end: bool,
    pub check_fail: Arc<dyn (Fn(&Self) -> bool) + Send + Sync>,
    pub do_health: Arc<dyn (Fn(&mut Self, &HitJudgment, &IngameScore)) + Send + Sync>,
}
impl HealthHelper {
    pub fn new() -> Self {
        let initial_health = 80.0;

        Self {
            current_health: initial_health,
            max_health: initial_health,
            initial_health,

            check_fail_at_end: false,
            check_fail: Arc::new(Self::default_check_fail),
            do_health: Arc::new(Self::default_do_health)
        }
    }

    pub fn is_dead(&self) -> bool {
        (self.check_fail.clone())(self)
    }

    pub fn get_ratio(&self) -> f32 {
        self.current_health / self.max_health
    }

    pub fn reset(&mut self) {
        self.current_health = self.initial_health;
    }

    #[inline]
    pub fn validate_health(&mut self) {
        if self.current_health < 0.0 { self.current_health = 0.0 }
        if self.current_health > self.max_health { self.current_health = self.max_health }
    }

}

// default fns
impl HealthHelper {
    fn default_check_fail(&self) -> bool {
        self.current_health <= 0.0
    }
    fn default_do_health(&mut self, j: &HitJudgment, _: &IngameScore) {
        self.current_health += j.health;
        self.validate_health()
    }
}


impl Default for HealthHelper {
    fn default() -> Self {
        Self::new()
    }
}