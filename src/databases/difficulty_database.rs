use crate::prelude::*;

const DIFFS_FILE:&'static str = "./diffs.json";
const TIMER:u64 = 1000;

async fn save(data: HashMap<PlayMode, HashMap<String, HashMap<String, f32>>>) {
    match serde_json::to_string(&data) {
        Ok(serialized) => {
            match tokio::fs::write(DIFFS_FILE, serialized).await {
                Ok(_) => println!("[Diffs] saved."),
                Err(e) => println!("[Diffs] error saving diffs: {}", e)
            }
        }
        Err(e) => println!("[Diffs] error serializing: {}", e)
    }
}
fn save_loop() {
    // println!("starting loop ======================================");
    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(TIMER)).await;

            if let Some(data) = read() {
                let current_data = DIFFICULTY_CALC_CACHE.read().clone();
                if data != current_data {
                    save(current_data).await
                }
            } else {
                let current_data = DIFFICULTY_CALC_CACHE.read().clone();
                save(current_data).await
            }
        }
    });
}

fn read() -> Option<HashMap<PlayMode, HashMap<String, HashMap<String, f32>>>> {
    if io::exists(DIFFS_FILE) {
        if let Ok(data) = std::fs::read(DIFFS_FILE) {
            if let Ok(data) = serde_json::from_slice(data.as_slice()) {
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

// temporarily a hashmap 
lazy_static::lazy_static! {
    /// mode, mods, map_hash = diff
    pub static ref DIFFICULTY_CALC_CACHE: Arc<RwLock<HashMap<PlayMode, HashMap<String, HashMap<String, f32>>>>> = {
        save_loop();
        if let Some(data) = read() {
            Arc::new(RwLock::new(data))
        } else {
            Arc::new(RwLock::new(HashMap::new()))
        }
    };
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