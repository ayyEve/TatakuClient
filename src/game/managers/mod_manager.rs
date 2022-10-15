use crate::prelude::*;

lazy_static::lazy_static! {
    static ref MOD_MANAGER: Arc<Mutex<ModManager>> = Arc::new(Mutex::new(ModManager::new()));
}

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(default)]
pub struct ModManager {
    /// use get/set_speed instead of direct access to this
    pub speed: u16,
    
    pub easy: bool,
    pub hard_rock: bool,
    pub autoplay: bool,
    pub nofail: bool,
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self {
            speed: 100,
            ..Self::default()
        }
    }
    
    pub async fn get<'a>() -> tokio::sync::MutexGuard<'a, Self> {
        MOD_MANAGER.lock().await
    }
}

// instance
impl ModManager {
    pub fn get_speed(&self) -> f32 {
        self.speed as f32 / 100.0
    }
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = (speed * 100.0).round() as u16;
        // error!("set speed: {speed} -> {}", self.speed);
    }

    fn mods_list(&self, include_speed: bool) -> String {
        let mut list = Vec::new();
        
        if self.easy { list.push("EZ".to_owned()) }
        if self.hard_rock { list.push("HR".to_owned()) }

        if self.nofail { list.push("NF".to_owned()) }
        if self.autoplay { list.push("AT".to_owned()) }

        let speed = self.get_speed();
        if include_speed && speed != 1.0 { list.push(format!("({:.2}x)", speed)) }

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


#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModManagerOld {
    /// use get/set_speed instead of direct access to this
    pub speed: u16,
    
    pub easy: bool,
    pub hard_rock: bool,
    pub autoplay: bool,
    pub nofail: bool,
}