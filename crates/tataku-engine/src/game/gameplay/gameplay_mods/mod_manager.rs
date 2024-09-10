use std::hash::Hash;
use crate::prelude::*;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq, Debug)]
#[derive(Reflect)]
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

    fn iter_mods(mode: &GameModeInfo) -> impl Iterator<Item=GameplayMod> {
        default_mod_groups()
            .into_iter()
            .chain(mode.mods.iter().map(GameplayModGroup::from_static))
            .flat_map(|m| m.mods)
    }

    pub fn mods_for_playmode(
        mode: &GameModeInfo
    ) -> Vec<GameplayMod> {
        Self::iter_mods(mode).collect()
    }
    pub fn mods_for_playmode_as_hashmap(
        mode: &GameModeInfo
    ) -> HashMap<String, GameplayMod> {
        Self::iter_mods(mode)
            .map(|m| (m.name.to_owned(), m))
            .collect()
    }

    pub fn short_mods_string(
        mods: &Vec<ModDefinition>, 
        none_if_empty: bool, 
        mode: &GameModeInfo,
    ) -> String {
        if mods.len() == 0 {
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
        mode: &GameModeInfo,
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
        // error!("set speed: {speed} -> {}", self.speed);
    }

    fn mods_list(
        &self, 
        include_speed: bool, 
        mode: &GameModeInfo,
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
        mode: &GameModeInfo,
    ) -> String {
        self.mods_list(true, mode)
    }
    pub fn mods_list_string_no_speed(
        &self, 
        mode: &GameModeInfo,
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


impl TryFrom<&TatakuValue> for ModManager {
    type Error = String;
    fn try_from(value: &TatakuValue) -> Result<Self, Self::Error> {
        let TatakuValue::Map(map) = value else { return Err(format!("Not a map")) };

        let Some(speed) = map.get("speed") else { return Err(format!("No speed entry")) };
        let TatakuValue::U32(speed) = &speed.value else { return Err(format!("speed entry is wrong type")) };

        let Some(mods) = map.get("mods") else { return Err(format!("No mods entry")) };
        let TatakuValue::List(mods) = &mods.value else { return Err(format!("Mods entry wrong type")) };

        Ok(Self {
            speed: GameSpeed::from_u16(*speed as u16),
            mods: mods.into_iter().map(|d|d.as_string()).collect()
        })
    }
}

impl Into<TatakuValue> for ModManager {
    fn into(self) -> TatakuValue {
        let mut map = HashMap::default();
        map.set_value("speed", TatakuVariable::new_game(TatakuValue::U32(self.speed.as_u16() as u32)));
        map.set_value("mods", TatakuVariable::new_game((TatakuVariableAccess::GameOnly, self.mods.iter().collect::<Vec<_>>())));
        TatakuValue::Map(map)
    }
}
