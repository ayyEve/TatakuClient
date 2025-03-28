use std::hash::Hash;
use crate::prelude::*;


pub const SPEED_STEP: u16 = 5;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq, Debug)]
#[derive(Reflect)]
#[reflect(display="debug")]
#[serde(default)]
pub struct ModManager {
    /// use get/set_speed instead of direct access to this
    pub speed: GameSpeed,
    pub mods: HashSet<String>,
}

// static 
impl ModManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn iter_mods(mode: &GamemodeInfo) -> impl Iterator<Item=GameplayMod> {
        default_mod_groups()
            .into_iter()
            .chain(mode.mods.iter().map(GameplayModGroup::from_static))
            .flat_map(|m| m.mods)
    }

    pub fn mods_for_playmode(
        mode: &GamemodeInfo
    ) -> Vec<GameplayMod> {
        Self::iter_mods(mode).collect()
    }
    pub fn mods_for_playmode_as_hashmap(
        mode: &GamemodeInfo
    ) -> HashMap<String, GameplayMod> {
        Self::iter_mods(mode)
            .map(|m| (m.name.to_owned(), m))
            .collect()
    }

    pub fn short_mods_string(
        mods: &[ModDefinition], 
        none_if_empty: bool, 
        mode: &GamemodeInfo,
    ) -> String {
        if mods.is_empty() {
            if none_if_empty { return "None".to_owned() }
            return String::new();
        }

        let ok_mods = Self::mods_for_playmode_as_hashmap(mode);

        let mut list = Vec::new();
        for m in mods.iter() {
            if let Some(m) = ok_mods.get(m.as_ref()) {
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


    pub fn map_mods_to_thing(
        &self, 
        mode: &GamemodeInfo,
    ) -> Vec<ModDefinition> {
        let ok_mods = ModManager::mods_for_playmode_as_hashmap(mode);

        self.mods.iter()
            .filter_map(|m| ok_mods.get(m))
            .map(|m| (*m).into())
            .collect()
    }
}

// instance
impl ModManager {
    pub fn get_speed(&self) -> f32 {
        self.speed.as_f32()
    }
    pub fn set_speed(&mut self, speed: impl Into<GameSpeed>) {
        self.speed = speed.into();
        let a = self.speed.as_u16();
        let speed_fixed = a - a % SPEED_STEP;
        self.speed = GameSpeed::from_u16(speed_fixed);
    }

    fn mods_list(
        &self, 
        include_speed: bool, 
        mode: &GamemodeInfo,
    ) -> String {
        let mod_groups = mode.mods;
        let mods = mod_groups
            .iter()
            .flat_map(|mg| mg.mods)
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

    pub fn mods_list_string(
        &self, 
        mode: &GamemodeInfo,
    ) -> String {
        self.mods_list(true, mode)
    }
    pub fn mods_list_string_no_speed(
        &self, 
        mode: &GamemodeInfo,
    ) -> String {
        self.mods_list(false, mode)
    }

    // inline helpers
    /// add a single mod
    pub fn with_mod(mut self, m: impl AsRef<str>) -> Self {
        self.add_mod(m);
        self
    }
    /// set all mods
    pub fn with_mods(mut self, mods: impl Iterator<Item=impl AsRef<str>>) -> Self {
        self.mods = mods.map(|i| i.as_ref().to_owned()).collect();
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
        self.mods.remove(m.as_ref());
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
        let mods_str = format!("{}{}", mods.join(""), self.speed.as_u16());
        md5(mods_str)
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
