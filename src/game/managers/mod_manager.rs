use crate::prelude::*;

lazy_static::lazy_static! {
    static ref MOD_MANAGER: Arc<Mutex<ModManager>> = Arc::new(Mutex::new(ModManager::new()));
}

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModManager {
    pub speed: f32,
    pub autoplay: bool,

    pub nofail: bool,
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            autoplay: false,
            nofail: false,
        }
    }
    
    pub fn get<'a>() -> MutexGuard<'a, Self> {
        MOD_MANAGER.lock()
    }
}

// instance
impl ModManager {}