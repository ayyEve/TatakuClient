use crate::prelude::*;

// a cache for storing all hitsounds that can be played in the map
// preventing "complex" math/code from happening every time a hitsound should be played
pub struct HitsoundHelper {
    list: Vec<HitsoundInstance>,

    current_index: usize,
    indices: Vec<(f32, usize)>,
}
impl HitsoundHelper {
    pub fn reset(&mut self) {
        self.current_index = 0;
    }
    pub fn add(
        &mut self, 
        time: f32, 

        don_hitsounds: Vec<Hitsound>,
        kat_hitsounds: Vec<Hitsound>,
        bigdon_hitsounds: Vec<Hitsound>,
        bigkat_hitsounds: Vec<Hitsound>,
    ) {
        
    }
}

#[derive(PartialEq, Eq)]
pub struct HitsoundInstance {
    don_hitsounds: Vec<Hitsound>,
    kat_hitsounds: Vec<Hitsound>,
    bigdon_hitsounds: Vec<Hitsound>,
    bigkat_hitsounds: Vec<Hitsound>,
}
impl HitsoundInstance {


    pub fn get(
        &self, 
        hit_type: HitType, 
        finisher: bool
    ) -> &[Hitsound] {
        match (hit_type, finisher) {
            (HitType::Don, false) => &self.don_hitsounds,
            (HitType::Kat, false) => &self.kat_hitsounds,
            (HitType::Don, true) => &self.bigdon_hitsounds,
            (HitType::Kat, true) => &self.bigkat_hitsounds,
        }
    }
}
