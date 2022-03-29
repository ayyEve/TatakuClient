use crate::prelude::*;

lazy_static::lazy_static! {
    static ref MOD_MANAGER: Arc<Mutex<ModManager>> = Arc::new(Mutex::new(ModManager::new()));
}

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModManager {
    pub speed: f32,
    
    pub easy: bool,
    pub hard_rock: bool,

    pub autoplay: bool,
    
    pub nofail: bool,
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            ..Self::default()
        }
    }
    
    pub fn get<'a>() -> MutexGuard<'a, Self> {
        MOD_MANAGER.lock()
    }
}

// instance
impl ModManager {
    pub fn mods_string(&self) -> String {
        let mut list = Vec::new();
        

        if self.easy {list.push("EZ".to_owned())}
        if self.hard_rock {list.push("HR".to_owned())}

        if self.nofail {list.push("NF".to_owned())}
        if self.autoplay {list.push("AT".to_owned())}

        if self.speed != 1.0 {list.push(format!("({:.2}x)", self.speed))}

        list.join(" ")
    }
}
