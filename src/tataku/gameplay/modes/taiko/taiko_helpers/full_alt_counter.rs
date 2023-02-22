use super::super::prelude::*;

#[derive(Default)]
pub struct FullAltCounter {
    // hits: HashMap<TaikoHit, usize>,
    last_hit: Option<TaikoHit>,
    // playmode: TaikoPlaymode
}
impl FullAltCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_hit(&mut self, hit: TaikoHit) -> bool {

        if self.last_hit.is_none() {
            self.last_hit = Some(hit);
            return true;
        }

        let is_left = Self::hit_is_left(hit);
        let last_is_left = Self::hit_is_left(self.last_hit.unwrap());
        self.last_hit = Some(hit);
        
        is_left != last_is_left
    }

    fn hit_is_left(hit: TaikoHit) -> bool {
        match hit {
            TaikoHit::LeftKat | TaikoHit::LeftDon => true,
            TaikoHit::RightDon | TaikoHit::RightKat => false,
        }
    }

}
