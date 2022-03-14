/*
 * this is a helper to store and retrieve scores, either online or local
 */

use ayyeve_piston_ui::prelude::Dropdown;

use crate::prelude::*;

lazy_static::lazy_static! {
    pub static ref SCORE_HELPER:Arc<RwLock<ScoreHelper>> = Arc::new(RwLock::new(ScoreHelper::new()));
}

pub struct ScoreHelper {
    pub current_method: ScoreRetreivalMethod,
}
impl ScoreHelper {
    pub fn new() -> Self {
        Self {
            current_method: ScoreRetreivalMethod::Local,
        }
    }

    pub fn get_scores(&self, map_hash: &String, playmode: &PlayMode) -> Arc<RwLock<ScoreLoaderHelper>> {
        let map_hash = map_hash.clone();
        let playmode = playmode.clone();
        let method = self.current_method;

        match method {
            ScoreRetreivalMethod::Local 
            | ScoreRetreivalMethod::LocalMods => {
                let scores = Arc::new(RwLock::new(ScoreLoaderHelper::new()));
                let scores_clone = scores.clone();
                tokio::spawn(async move {
                    let mut local_scores = get_scores(&map_hash, playmode);

                    if method.filter_by_mods() {
                        let mods = ModManager::get().clone();
                        let mods_string = Some(serde_json::to_string(&mods).unwrap());
                        local_scores.retain(|s| s.mods_string == mods_string);
                    }
                    let mut thing = scores_clone.write();
                    thing.scores = local_scores;
                    thing.done = true;
                });
                
                scores
            },
            ScoreRetreivalMethod::Global
            | ScoreRetreivalMethod::GlobalMods => {
                let scores = Arc::new(RwLock::new(ScoreLoaderHelper::new()));
                
                let scores_clone = scores.clone();
                tokio::spawn(async move {
                //     let mut online_scores = get_online_scores(&map_hash, playmode);

                //     if method.filter_by_mods() {
                //         let mods = ModManager::get().clone();
                //         let mods_string = Some(serde_json::to_string(&mods).unwrap());
                //         online_scores.retain(|s| s.mods_string == mods_string);
                //     }

                    let mut thing = scores_clone.write();
                //     thing.scores = local_scores;
                    thing.done = true;
                });
                
                //TODO: this
                
                scores.write().done = true;
                scores
            },
        }
    }
}

/// helper for retreiving scores from online (async)
pub struct ScoreLoaderHelper {
    pub scores: Vec<Score>,
    pub done: bool
}
impl ScoreLoaderHelper {
    pub fn new() -> Self {
        Self {
            scores: Vec::new(),
            done: false
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Dropdown)]
pub enum ScoreRetreivalMethod {
    Local,
    LocalMods,
    Global,
    GlobalMods,
    // Friends,
    // FriendsMods
}
impl ScoreRetreivalMethod {
    pub fn filter_by_mods(&self) -> bool {
        use ScoreRetreivalMethod::*;
        match self {
            Local 
            // | Friends
            | Global => false,

            LocalMods
            // | FriendsMods
            | GlobalMods => true,
        }
    }
}