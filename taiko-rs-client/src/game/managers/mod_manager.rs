use crate::sync::*;

lazy_static::lazy_static! {
    static ref MOD_MANAGER: Arc<Mutex<ModManager>> = Arc::new(Mutex::new(ModManager::new()));
}

#[derive(Clone, Default)]
pub struct ModManager {
    pub speed: f32,
    pub autoplay: bool,
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            autoplay: false,
        }
    }
    
    pub fn get<'a>() -> MutexGuard<'a, Self> {
        MOD_MANAGER.lock()
    }
}

// instance
impl ModManager {
}