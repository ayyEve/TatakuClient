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
        if MOD_MANAGER.is_locked() {
            println!("MOD MANAGER LOCKED");
        }
        MOD_MANAGER.lock()
    }
}

// instance
impl ModManager {
    pub fn mods_string(&self) -> String {
        let mut list = Vec::new();
        
        if self.nofail {list.push("NF".to_owned())}
        if self.autoplay {list.push("AT".to_owned())}
        if self.speed != 1.0 {list.push(format!("({:.2}x)", self.speed))}

        list.join(" ")
    }
}