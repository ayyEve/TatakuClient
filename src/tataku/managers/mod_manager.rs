use std::hash::Hash;
use crate::prelude::*;

pub type ModManagerHelper = GlobalValue<ModManager>;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq, Debug)]
#[serde(default)]
pub struct ModManager {
    /// use get/set_speed instead of direct access to this
    pub speed: GameSpeed,
    
    pub mods: HashSet<String>
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self::default()
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

    pub fn mods_for_playmode(playmode: &String) -> Vec<GameplayMod> {
        let Some(info) = get_gamemode_info(playmode) else { return Vec::new() };

        default_mod_groups()
            .into_iter()
            .chain(info.get_mods().into_iter())
            .map(|m|m.mods)
            .flatten()
            .collect::<Vec<_>>()
    }
    pub fn mods_for_playmode_as_hashmap(playmode: &String) -> HashMap<String, GameplayMod> {
        let Some(info) = get_gamemode_info(playmode) else { return HashMap::new() };

        default_mod_groups()
            .into_iter()
            .chain(info.get_mods().into_iter())
            .map(|m| m.mods)
            .flatten()
            .map(|m| (m.name.to_owned(), m))
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
                list.push(m.short_name)
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
        self.speed.as_f32()
    }
    pub fn set_speed(&mut self, speed: impl Into<GameSpeed>) {
        self.speed = speed.into();
        // error!("set speed: {speed} -> {}", self.speed);
    }

    fn mods_list(&self, include_speed: bool, mode: &String) -> String {
        let Some(gamemode_info) = get_gamemode_info(mode) else { return String::new() };
        let mod_groups = gamemode_info.get_mods();
        let mods = mod_groups
            .iter()
            .map(|mg| &mg.mods)
            .flatten()
            .map(|m| (m.name, m))
            .collect::<HashMap<_,_>>();

        let mut list = self.mods
            .iter()
            .filter_map(|id| mods.get(&**id))
            .map(|m| m.short_name.to_owned())
            .collect::<Vec<_>>();


        if include_speed && !self.speed.is_default() { list.push(format!("({:.2}x)", self.get_speed())) }

        list.join(" ")
    }

    fn mods_sorted(&self) -> Vec<String> {
        let mut mods = self.mods.clone().into_iter().collect::<Vec<_>>();
        mods.sort();
        mods
    }

    pub fn mods_list_string(&self, mode: &String) -> String {
        self.mods_list(true, mode)
    }
    pub fn mods_list_string_no_speed(&self, mode: &String) -> String {
        self.mods_list(false, mode)
    }

    // inline helpers
    /// add a single mod
    pub fn with_mod(mut self, m: impl AsRef<str>) -> Self {
        self.add_mod(m);
        self
    }
    /// set all mods
    pub fn with_mods(mut self, mods: HashSet<String>) -> Self {
        self.mods = mods;
        self
    }
    /// set the speed
    pub fn with_speed(mut self, speed: impl Into<GameSpeed>) -> Self {
        self.set_speed(speed);
        self
    }

    /// add a mod, returns if the mod was already added
    pub fn add_mod(&mut self, m: impl AsRef<str>) -> bool {
        self.mods.insert(m.as_ref().to_owned())
    }
    /// remove a mod
    pub fn remove_mod(&mut self, m: impl AsRef<str>) {
        self.mods.remove(&m.as_ref().to_owned());
    }
    // toggle a mod, returns if the mod is now enabled or not
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
    
    /// returns if a mod is enabled
    pub fn has_mod(&self, m: impl AsRef<str>) -> bool {
        self.mods.contains(m.as_ref())
    }


    // common mods
    /// is nofail enabled
    pub fn has_nofail(&self) -> bool {
        self.has_mod(NoFail)
    }
    /// is sudden death enabled
    pub fn has_sudden_death(&self) -> bool {
        self.has_mod(SuddenDeath)
    }
    /// is perfect enabled
    pub fn has_perfect(&self) -> bool {
        self.has_mod(Perfect)
    }
    /// is autoplay enabled
    pub fn has_autoplay(&self) -> bool {
        self.has_mod(Autoplay)
    }

    pub fn as_md5(&self) -> Md5Hash {
        let mods = self.mods_sorted();
        let mods_str = format!("{}{}", mods.join(""), self.speed.as_u16().to_string());
        md5(mods_str)
        // u128::from_str_radix(&md5(mods_str).to_string(), 16).unwrap_or_default()
    }
}

// lets pretend this is correct for now
impl Hash for ModManager {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.speed.hash(state);
        let mods = self.mods_sorted();
        mods.hash(state);
    }
}


impl TryFrom<&CustomElementValue> for ModManager {
    type Error = String;
    fn try_from(value: &CustomElementValue) -> Result<Self, Self::Error> {
        let CustomElementValue::Map(map) = value else { return Err(format!("Not a map")) };

        let Some(speed) = map.get("speed") else { return Err(format!("No speed entry")) };
        let CustomElementValue::U32(speed) = speed else { return Err(format!("speed entry is wrong type")) };

        let Some(mods) = map.get("mods") else { return Err(format!("No mods entry")) };
        let CustomElementValue::List(mods) = mods else { return Err(format!("Mods entry wrong type")) };

        Ok(Self {
            speed: GameSpeed::from_u16(*speed as u16),
            mods: mods.into_iter().map(CustomElementValue::as_string).collect()
        })
    }
}

impl Into<CustomElementValue> for ModManager {
    fn into(self) -> CustomElementValue {
        let mut map = CustomElementMapHelper::default();
        map.set("speed", CustomElementValue::U32(self.speed.as_u16() as u32));
        map.set("mods", self.mods.iter().collect::<Vec<_>>());
        map.finish()
    }
}