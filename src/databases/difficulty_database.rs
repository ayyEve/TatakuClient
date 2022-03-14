use crate::prelude::*;
// temporarily a hashmap 

lazy_static::lazy_static! {
    /// mode, mods, map_hash = diff
    pub static ref DIFFICULTY_CALC_CACHE: Arc<RwLock<HashMap<PlayMode, HashMap<String, HashMap<String, f32>>>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn insert_diff(map_hash: &String, playmode: &PlayMode, mods: &ModManager, diff: f32) {
    let mut lock = DIFFICULTY_CALC_CACHE.write();
    if !lock.contains_key(playmode) {
        lock.insert(playmode.clone(), HashMap::new());
    }

    let mods_key = serde_json::to_string(mods).unwrap();
    let thing = lock.get_mut(playmode).unwrap();
    if !thing.contains_key(map_hash) {
        thing.insert(map_hash.clone(), HashMap::new());
    }

    let thing2 = thing.get_mut(map_hash).unwrap();
    thing2.insert(mods_key.clone(), diff);
}

pub fn get_diff(map_hash: &String, playmode: &PlayMode, mods: &ModManager) -> Option<f32> {
    let lock = DIFFICULTY_CALC_CACHE.read();
    if !lock.contains_key(playmode) {
        return None;
    }

    let mods_key = serde_json::to_string(mods).unwrap();
    let thing = lock.get(playmode).unwrap();
    if !thing.contains_key(map_hash) {
        return None;
    }
    
    let thing2 = thing.get(map_hash).unwrap();
    thing2.get(&mods_key).and_then(|d|Some(*d))
}