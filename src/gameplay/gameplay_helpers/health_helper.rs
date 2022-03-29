#![allow(dead_code)]

#[derive(Clone, Default)]
pub struct HealthHelper {
    pub max_health: f32,
    pub current_health: f32,

    pub damage_amount: f32,
    pub heal_amount: f32,
}
impl HealthHelper {
    pub fn new(_hp_val: Option<f32>) -> Self {
        let max_health = 80.0;

        Self {
            max_health,
            current_health: max_health,

            damage_amount: 10.0,
            heal_amount: 3.0
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current_health <= 0.0
    }

    pub fn get_ratio(&self) -> f32 {
        self.current_health / self.max_health
    }

    pub fn reset(&mut self) {
        self.current_health = self.max_health;
    }

    #[inline]
    pub fn validate_health(&mut self) {
        if self.current_health < 0.0 {self.current_health = 0.0}
        if self.current_health > self.max_health {self.current_health = self.max_health}
    }


    pub fn take_damage(&mut self) {
        self.current_health -= self.damage_amount;
        self.validate_health();
    }
    pub fn give_life(&mut self) {
        self.current_health += self.heal_amount;
        self.validate_health();
    }
    pub fn give_extra_life(&mut self) {
        self.current_health += self.heal_amount * 1.5;
        self.validate_health();
    }
}
