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
    
    pub async fn get<'a>() -> tokio::sync::MutexGuard<'a, Self> {
        MOD_MANAGER.lock().await
    }
}

// instance
impl ModManager {
    fn mods_list(&self, include_speed: bool) -> String {
        let mut list = Vec::new();
        
        if self.easy { list.push("EZ".to_owned()) }
        if self.hard_rock { list.push("HR".to_owned()) }

        if self.nofail { list.push("NF".to_owned()) }
        if self.autoplay { list.push("AT".to_owned()) }

        if include_speed && self.speed != 1.0 { list.push(format!("({:.2}x)", self.speed)) }

        list.join(" ")
    }

    pub fn mods_list_string(&self) -> String {
        self.mods_list(true)
    }
    pub fn mods_list_string_no_speed(&self) -> String {
        self.mods_list(false)
    }

    pub fn as_json(&self) -> String {
        serde_json::to_string(self).expect("error converting mods to json string")
    }
}
