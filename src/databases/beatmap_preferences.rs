/**
 * This is used to store map and mode preferences. 
 * this should probably be improved, but i wanted to add it quickly
 */

use crate::prelude::*;

const MAP_PREFS_FILE:&'static str = "map_prefs.json";
const MAP_MODE_PREFS_FILE:&'static str = "map_mode_prefs.json";
const TIMER:u64 = 1000;

lazy_static::lazy_static! {
    static ref BEATMAP_PREFERENCES: Arc<RwLock<HashMap<String, BeatmapPreferences>>> = {
        let a = Arc::new(RwLock::new(read_map_prefs().unwrap_or_default()));
        save_loop();
        a
    };
    static ref BEATMAP_MODE_PREFERENCES: Arc<RwLock<HashMap<String, HashMap<PlayMode, BeatmapPlaymodePreferences>>>> = Arc::new(RwLock::new(read_map_mode_prefs().unwrap_or_default()));
}

async fn save_map_prefs(data: HashMap<String, BeatmapPreferences>) {
    match serde_json::to_string(&data) {
        Ok(serialized) => {
            match tokio::fs::write(MAP_PREFS_FILE, serialized).await {
                Ok(_) => println!("[MapPrefs] saved."),
                Err(e) => println!("[MapPrefs] error saving diffs: {}", e)
            }
        }
        Err(e) => println!("[MapPrefs] error serializing: {}", e)
    }
}
async fn save_map_mode_prefs(data: HashMap<String, HashMap<PlayMode, BeatmapPlaymodePreferences>>) {
    match serde_json::to_string(&data) {
        Ok(serialized) => {
            match tokio::fs::write(MAP_MODE_PREFS_FILE, serialized).await {
                Ok(_) => println!("[MapModePrefs] saved."),
                Err(e) => println!("[MapModePrefs] error saving diffs: {}", e)
            }
        }
        Err(e) => println!("[MapModePrefs] error serializing: {}", e)
    }
}
fn save_loop() {
    // println!("starting loop ======================================");
    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_millis(TIMER)).await;

            // map prefs
            if let Some(data) = read_map_prefs() {
                let current_data = BEATMAP_PREFERENCES.read().clone();
                if data != current_data {
                    save_map_prefs(current_data).await;
                }
            } else {
                let current_data = BEATMAP_PREFERENCES.read().clone();
                save_map_prefs(current_data).await;
            }

            // mode prefs
            if let Some(data) = read_map_mode_prefs() {
                let current_data = BEATMAP_MODE_PREFERENCES.read().clone();
                if data != current_data {
                    save_map_mode_prefs(current_data).await;
                }
            } else {
                let current_data = BEATMAP_MODE_PREFERENCES.read().clone();
                save_map_mode_prefs(current_data).await;
            }
        }
    });
}

fn read_map_prefs() -> Option<HashMap<String, BeatmapPreferences>> {
    if let Ok(data) = std::fs::read(MAP_PREFS_FILE) {
        if let Ok(data) = serde_json::from_slice(data.as_slice()) {
            Some(data)
        } else {
            None
        }
    } else {
        None
    }
}

fn read_map_mode_prefs() -> Option<HashMap<String, HashMap<PlayMode, BeatmapPlaymodePreferences>>> {
    if let Ok(data) = std::fs::read(MAP_MODE_PREFS_FILE) {
        if let Ok(data) = serde_json::from_slice(data.as_slice()) {
            Some(data)
        } else {
            None
        }
    } else {
        None
    }
}



#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BeatmapPreferences {
    pub audio_offset: f32,
    // pub background_video: bool,
    // pub storyboard: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BeatmapPlaymodePreferences {
    pub scroll_speed: f32,
}
impl Default for BeatmapPlaymodePreferences {
    fn default() -> Self {
        Self { 
            scroll_speed: 1.0,
        }
    }
}

pub fn get_beatmap_prefs(map_hash:&String) -> BeatmapPreferences {
    let prefs = BEATMAP_PREFERENCES
        .read()
        .get(map_hash)
        .cloned()
        .unwrap_or_default();

    prefs
}
pub fn get_beatmap_mode_prefs(map_hash:&String, playmode:&PlayMode) -> BeatmapPlaymodePreferences {
    let mode_prefs = if let Some(modes) = BEATMAP_MODE_PREFERENCES
        .read().get(map_hash) {
            modes
            .get(playmode)
            .cloned()
            .unwrap_or_default()
        } else {
            Default::default()
        };
    
    mode_prefs
}

pub fn save_beatmap_prefs(map_hash:&String, prefs: BeatmapPreferences) {
    BEATMAP_PREFERENCES.write().insert(map_hash.clone(), prefs);
}
pub fn save_beatmap_playmode_prefs(map_hash:&String, playmode: &PlayMode, prefs: BeatmapPlaymodePreferences) {
    let mut lock = BEATMAP_MODE_PREFERENCES.write();
    if !lock.contains_key(map_hash) {
        lock.insert(map_hash.clone(), HashMap::new());
    }

    if let Some(modes) = lock.get_mut(map_hash) {
        modes.insert(playmode.clone(), prefs);
    } else {
        panic!("what")
    }
}