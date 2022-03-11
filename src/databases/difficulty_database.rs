use crate::prelude::*;
// temporarily a hashmap 

lazy_static::lazy_static! {
    pub static ref DIFFICULTY_CALC_CACHE: Arc<RwLock<HashMap<PlayMode, HashMap<String, f64>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn insert_diff(playmode: &PlayMode, mods: &ModManager, diff: f64) {
    let mut lock = DIFFICULTY_CALC_CACHE.write();
    if !lock.contains_key(playmode) {
        lock.insert(playmode.clone(), HashMap::new());
    }

    let mods_key = serde_json::to_string(mods).unwrap();
    let thing = lock.get_mut(playmode).unwrap();
    thing.insert(mods_key.clone(), diff);
}

pub fn get_diff(playmode: &PlayMode, mods: &ModManager) -> f64 {
    let lock = DIFFICULTY_CALC_CACHE.read();
    if !lock.contains_key(playmode) {
        return 0.0;
    }

    let mods_key = serde_json::to_string(mods).unwrap();
    let thing = lock.get(playmode).unwrap();
    *thing.get(&mods_key).unwrap_or(&0.0)
}