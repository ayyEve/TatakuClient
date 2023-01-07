use std::hash::Hash;
use crate::prelude::*;

pub type ModManagerHelper = GlobalValue<ModManager>;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq, Debug)]
#[serde(default)]
pub struct ModManager {
    /// use get/set_speed instead of direct access to this
    pub speed: u16,
    
    pub mods: HashSet<String>
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self {
            speed: 100,
            ..Self::default()
        }
    }
    
    pub fn get_mut() -> GlobalValueMut<Self> {
        GlobalValueManager::get_mut::<Self>().unwrap()
    }
    pub fn get() -> Arc<Self> {
        GlobalValueManager::get::<Self>().unwrap()
    }
    pub fn get_cloned() -> Self {
        GlobalValueManager::get::<Self>().unwrap().as_ref().clone()
    }

    pub fn mods_for_playmode(playmode: &String) -> Vec<Box<dyn GameplayMod>> {
        let Some(info) = get_gamemode_info(playmode) else { return Vec::new() };

        default_mod_groups()
            .into_iter()
            .chain(info.get_mods().into_iter())
            .map(|m|m.mods)
            .flatten()
            .collect::<Vec<_>>()
    }
    pub fn mods_for_playmode_as_hashmap(playmode: &String) -> HashMap<String, Box<dyn GameplayMod>> {
        let Some(info) = get_gamemode_info(playmode) else { return HashMap::new() };

        default_mod_groups()
            .into_iter()
            .chain(info.get_mods().into_iter())
            .map(|m|m.mods)
            .flatten()
            .map(|m|(m.name().to_owned(), m))
            .collect()
    }

    pub fn short_mods_string(mods: HashSet<String>, none_if_empty: bool, playmode: &String) -> String {
        if mods.len() == 0 {
            if none_if_empty { return "None".to_owned() }
            return String::new();
        }

        let ok_mods = Self::mods_for_playmode_as_hashmap(playmode);

        let mut list = Vec::new();
        for m in mods.iter() {
            if let Some(m) = ok_mods.get(m) {
                list.push(m.short_name())
            }
        }


        // //TODO: sort this somehow?
        // let mut list = Vec::new();

        // for m in mods.iter() {
        //     match &**m {
        //         "easy" => list.push("EZ".to_owned()),
        //         "autoplay" => list.push("AT".to_owned()),

        //         // ignore empty
        //         _ if m.trim().is_empty() => {}

        //         // split by _, and capitalize the first letter in each split, and join without spaces
        //         // no_fail -> NF (No_Fail)
        //         // this_is_a_mod -> TIAM
        //         m => {
        //             list.push(m.split("_").map(|s|s.chars().next().unwrap().to_uppercase().to_string()).collect::<Vec<String>>().join(""))
        //         },
        //     }
        // }

        list.join(" ")
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

        for m in self.mods.iter() {
            match &**m {
                "easy" => list.push("EZ".to_owned()),
                "autoplay" => list.push("AT".to_owned()),

                m => list.push(m.split("_").map(|s|s.chars().next().unwrap().to_uppercase().to_string()).collect::<Vec<String>>().join("")),
            }
        }

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

    // inline helper
    pub fn with_mod(mut self, m: impl AsRef<str>) -> Self {
        self.add_mod(m);
        self
    }
    pub fn with_speed(mut self, speed: u16) -> Self {
        self.speed = speed;
        self
    }
    pub fn with_speed_f32(mut self, speed: f32) -> Self {
        self.set_speed(speed);
        self
    }


    pub fn add_mod(&mut self, m: impl AsRef<str>) -> bool {
        self.mods.insert(m.as_ref().to_owned())
    }
    pub fn remove_mod(&mut self, m: impl AsRef<str>) {
        self.mods.remove(&m.as_ref().to_owned());
    }
    pub fn toggle_mod(&mut self, m: impl AsRef<str>) -> bool {
        let m = m.as_ref().to_owned();
        if self.has_mod(&m) {
            self.remove_mod(&m);
            false
        } else {
            self.add_mod(&m);
            true
        }
    }
    
    pub fn has_mod(&self, m: impl AsRef<str>) -> bool {
        self.mods.contains(m.as_ref())
    }


    // common mods
    pub fn has_nofail(&self) -> bool {
        self.has_mod(NoFail.name())
    }
    pub fn has_sudden_death(&self) -> bool {
        self.has_mod(SuddenDeath.name())
    }
    pub fn has_perfect(&self) -> bool {
        self.has_mod(Perfect.name())
    }
    pub fn has_autoplay(&self) -> bool {
        self.has_mod(Autoplay.name())
    }


}

// lets pretend this is correct for now
impl Hash for ModManager {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.speed.hash(state);
        let mut mods = self.mods.clone().into_iter().collect::<Vec<_>>();
        mods.sort();
        mods.hash(state);
    }
}



#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OldModManager {
    /// use get/set_speed instead of direct access to this
    pub speed: Option<u16>,
    
    pub easy: bool,
    pub hard_rock: bool,
    pub autoplay: bool,
    pub nofail: bool,
}
